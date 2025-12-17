use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{ExecutableCommand, cursor, event::PopKeyboardEnhancementFlags, terminal};

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

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    spawn_coordination_threads(tx);

    let (mut game_state, mut renderer) = create_world()?;

    let kb_enhanced = renderer.terminal_raw_mode()?;

    if let Err(e) = renderer.render(&mut game_state) {
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
        match rx.recv() {
            Ok(event) => match event {
                GameEvent::Quit => break,
                other => {
                    tick_pending |= handle_event(other, &mut renderer, &mut game_state);
                }
            },
            Err(_) => continue,
        };

        // Drain any queued events; fold multiple ticks into a single step
        while let Ok(event) = rx.try_recv() {
            match event {
                GameEvent::Quit => {
                    // Exit immediately on quit
                    return Ok(());
                }
                other => {
                    tick_pending |= handle_event(other, &mut renderer, &mut game_state);
                }
            }
        }

        if tick_pending {
            if game_state.paused {
                renderer.render(&mut game_state);
                continue;
            } else if game_state.game_over {
                continue;
            }

            let now = Instant::now();
            let mut dt = now.duration_since(last_frame_time);
            last_frame_time = now;
            // Clamp dt to reduce perceived speed changes when we fall behind
            dt = dt.min(max_dt);

            movement_system(dt.max(fixed_dt).min(max_dt), &mut game_state)?;

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
