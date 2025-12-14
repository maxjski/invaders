use std::error::Error;
use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, cursor,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    queue,
    terminal::{self, Clear, ClearType},
};

use hecs::World;

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

    let mut stdout = stdout();

    // Enter raw mode, ask terminal to report key releases (if supported), and hide cursor
    terminal::enable_raw_mode()?;
    let kb_flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
    let kb_enhanced = stdout
        .execute(PushKeyboardEnhancementFlags(kb_flags))
        .is_ok();
    queue!(stdout, cursor::Hide)?;

    // World setup
    let world = World::new();

    // Each frame is a list of lines
    let mut game_state = GameState { world };

    game_state.world.spawn((
        Player,
        Position { x: 55, y: 7 },
        PrevPosition { x: 55, y: 7 },
        Velocity {
            speed: 60.0,
            move_accumulator: 0.0,
            direction: Direction::None,
        },
        Renderable {
            sprite_top: "⣆⡜⣛⢣⣠",
            sprite_bottom: "⣿⣿⣿⣿⣿",
            width: 5,
        },
    ));

    let mut renderer = Render {
        stdout,
        wsize: terminal::window_size()?,
        wsize_updated: true,
    };

    if let Err(e) = renderer.render(&mut game_state.world) {
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
                    tick_pending |= handle_event(other, &mut renderer, &mut game_state.world);
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
                    tick_pending |= handle_event(other, &mut renderer, &mut game_state.world);
                }
            }
        }

        if tick_pending {
            let now = Instant::now();
            let mut dt = now.duration_since(last_frame_time);
            last_frame_time = now;
            // Clamp dt to reduce perceived speed changes when we fall behind
            dt = dt.min(max_dt);

            movement_system(dt.max(fixed_dt).min(max_dt), &mut game_state.world);
            renderer.render(&mut game_state.world)?;
            // renderer
            //     .game_state
            //     .player
            //     .update(dt.max(fixed_dt).min(max_dt));
            // renderer.game_state.player_updated = true;

            match renderer.render(&mut game_state.world) {
                Ok(_) => continue,
                Err(_) => {
                    break;
                }
            }
        }
    }

    // Disable keyboard enhancement (if enabled), show cursor again, and disable raw mode before exiting
    if kb_enhanced {
        let _ = renderer.stdout.execute(PopKeyboardEnhancementFlags);
    }
    renderer.stdout.execute(Clear(ClearType::All))?;
    renderer.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
