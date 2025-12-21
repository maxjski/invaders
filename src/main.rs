use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use crossterm::{ExecutableCommand, cursor, event::PopKeyboardEnhancementFlags, terminal};

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

mod components;
mod events;
mod render;
mod state;
mod systems;
use crate::components::*;
use crate::events::*;
use crate::render::*;
use crate::state::*;
use crate::systems::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Networking

    let (tx, mut rx) = mpsc::unbounded_channel();

    spawn_coordination_threads(&tx);

    let (mut game_state, mut renderer) = create_world()?;

    let kb_enhanced = renderer.terminal_raw_mode()?;

    if let Err(e) = renderer.render_main_menu(&mut game_state) {
        // We drop errors to keep and return the game_state.render() error instead
        if kb_enhanced {
            let _ = renderer.stdout.execute(PopKeyboardEnhancementFlags);
        }
        let _ = renderer.stdout.execute(cursor::Show);
        let _ = terminal::disable_raw_mode();

        return Err(e);
    }

    let mut last_frame_time = Instant::now();
    let max_dt = Duration::from_millis(20); // clamp to avoid speed spikes
    let fixed_dt = Duration::from_millis(16);

    loop {
        // Block until at least one event arrives
        let mut tick_pending = false;
        match rx.recv().await {
            Some(GameEvent::Quit) => match game_state.main_menu.screen {
                Screen::Game => {
                    game_state.exit_to_menu();
                    // game_state.main_menu.screen = Screen::Main;
                    // game_state.request_clear_render = true;
                    // game_state.restart_notifier = true;
                }
                Screen::Hosting => {
                    game_state.exit_to_menu();
                    // game_state.main_menu.screen = Screen::Main;
                    // game_state.request_clear_render = true;
                    // game_state.restart_notifier = true;
                    // game_state.networking.listener_task = Option::None;
                    // game_state.networking.peer = Option::None;
                }
                Screen::Joining => {
                    game_state.exit_to_menu();
                }
                Screen::Main => {
                    break;
                }
            },
            Some(event) => tick_pending |= handle_event(event, &mut renderer, &mut game_state),
            _ => break,
        };

        // Drain any queued events; fold multiple ticks into a single step
        while let Ok(event) = rx.try_recv() {
            match event {
                GameEvent::Quit => {
                    // Exit immediately on quit
                    renderer.terminal_disable_raw(kb_enhanced)?;

                    return Ok(());
                }
                other => {
                    tick_pending |= handle_event(other, &mut renderer, &mut game_state);
                }
            }
        }

        if tick_pending {
            if !game_state.networking.stay_online {
                if let Some(handle) = game_state.networking.connection_task.take() {
                    handle.abort();
                    game_state.networking.connection_task = Option::None;
                }
            } else if game_state.networking.connection_task.is_none() {
                if game_state.networking.host {
                    let tx_net = tx.clone();
                    let task = tokio::spawn(async move {
                        match TcpListener::bind("127.0.0.1:23471").await {
                            Ok(listener) => {
                                if let Ok((_socket, addr)) = listener.accept().await {
                                    let _ = tx_net.send(GameEvent::ClientConnected(addr));
                                }
                            }
                            Err(_) => { /* Handle bind error if necessary */ }
                        }
                    });
                    game_state.networking.connection_task = Some(task);
                } else {
                    let tx_net = tx.clone();
                    let task = tokio::spawn(async move {
                        match TcpStream::connect("127.0.0.1:23471").await {
                            Ok(mut stream) => {
                                // if let Ok((_socket, addr)) = listener.accept().await {
                                //     let _ = tx_net.send(GameEvent::ClientConnected(addr));
                                // }
                                let socket =
                                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 23471);
                                let _ = tx_net.send(GameEvent::ClientConnected(socket));
                            }
                            Err(_) => { /* Handle bind error if necessary */ }
                        }
                    });
                    game_state.networking.connection_task = Some(task);
                }
            }

            match game_state.main_menu.screen {
                Screen::Hosting => {
                    renderer.render_host_menu(&mut game_state)?;
                    continue;
                }
                Screen::Main => {
                    renderer.render_main_menu(&mut game_state)?;
                    continue;
                }
                Screen::Joining => {
                    renderer.render_join_menu(&mut game_state)?;
                    continue;
                }
                _ => (),
            }

            if game_state.restart_notifier {
                (game_state, renderer) = restart_world(game_state.high_score)?;
                continue;
            }

            if game_state.pause_notifier {
                // renderer.render(&mut game_state)?;
                if game_state.paused {
                    game_state.paused = false;
                    renderer.erase_pause()?;
                } else {
                    game_state.paused = true;
                    renderer.draw_pause()?;
                }
                game_state.pause_notifier = false;
            }

            if game_state.game_over_notifier {
                if game_state.game_over {
                    game_state.game_over = false;
                    renderer.erase_game_over()?;
                } else {
                    game_state.game_over = true;
                    renderer.draw_menu_items(
                        game_state.score,
                        game_state.high_score,
                        game_state.player_lives,
                        false,
                    )?;
                    renderer.draw_game_over(game_state.score, game_state.high_score)?;

                    if game_state.score > game_state.high_score {
                        game_state.high_score = game_state.score;
                    }
                }
                game_state.game_over_notifier = false;
            }
            if game_state.game_over || game_state.paused {
                continue;
            }

            let now = Instant::now();
            let mut dt = now.duration_since(last_frame_time);
            last_frame_time = now;
            // Clamp dt to reduce perceived speed changes when we fall behind
            dt = dt.min(max_dt);

            process_tick(dt.max(fixed_dt).min(max_dt), &mut game_state)?;

            match renderer.render(&mut game_state) {
                Ok(_) => continue,
                Err(_) => {
                    break;
                }
            }
        }
    }

    // Disable keyboard enhancement (if enabled), show cursor again, and disable raw mode before exiting
    renderer.terminal_disable_raw(kb_enhanced)?;
    Ok(())
}
