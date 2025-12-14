use crate::{Direction, Player, Render, Velocity, World};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{Event, KeyCode, read},
    terminal,
};

pub enum GameEvent {
    ResizeGame,
    Tick,
    Quit,
    MovePlayerLeft,
    MovePlayerRight,
    MovePlayerStop,
}

pub fn handle_event(event: GameEvent, renderer: &mut Render, world: &mut World) -> bool {
    match event {
        GameEvent::ResizeGame => {
            renderer.wsize_updated = true;
            if let Ok(size) = terminal::window_size() {
                renderer.wsize = size;
            }
            let _ = renderer.render(world); // render immediately to reflect new bounds
            false
        }
        GameEvent::MovePlayerLeft => {
            for (_, vel) in world.query_mut::<&mut Velocity>().with::<&Player>() {
                vel.direction = Direction::Left;
            }
            false
        }
        GameEvent::MovePlayerRight => {
            for (_, vel) in world.query_mut::<&mut Velocity>().with::<&Player>() {
                vel.direction = Direction::Right;
            }
            false
        }
        GameEvent::MovePlayerStop => {
            for (_, vel) in world.query_mut::<&mut Velocity>().with::<&Player>() {
                vel.direction = Direction::None;
            }
            false
        }
        GameEvent::Tick => true,
        GameEvent::Quit => false,
    }
}

pub fn spawn_coordination_threads(tx: Sender<GameEvent>) {
    let tx_tick = tx.clone();
    thread::spawn(move || {
        loop {
            if tx_tick.send(GameEvent::Tick).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(16));
        }
    });

    // handle events
    thread::spawn(move || {
        loop {
            match read() {
                Ok(event) => match event {
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Char('q') {
                            match tx.send(GameEvent::Quit) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('a') && key_event.is_press() {
                            match tx.send(GameEvent::MovePlayerLeft) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('d') && key_event.is_press() {
                            match tx.send(GameEvent::MovePlayerRight) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if (key_event.code == KeyCode::Char('a')
                            || key_event.code == KeyCode::Char('d'))
                            && key_event.is_release()
                        {
                            match tx.send(GameEvent::MovePlayerStop) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        }
                    }
                    Event::Resize(_, _) => match tx.send(GameEvent::ResizeGame) {
                        Ok(_) => continue,
                        Err(_) => break,
                    },
                    _ => continue,
                },
                Err(_) => {
                    continue;
                }
            }
        }
    });
}
