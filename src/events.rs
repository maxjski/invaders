use crate::{
    Direction, GameState, Player, PlayerProjectile, Position, PrevPosition, Render, Renderable,
    Velocity,
};
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
    PlayerShoot,
    Pause,
}

pub fn handle_event(event: GameEvent, renderer: &mut Render, game_state: &mut GameState) -> bool {
    match event {
        GameEvent::ResizeGame => {
            renderer.wsize_updated = true;
            if let Ok(size) = terminal::window_size() {
                renderer.wsize = size;
            }
            let _ = renderer.render(game_state); // render immediately to reflect new bounds
            false
        }
        GameEvent::PlayerShoot => {
            if game_state.player_projectile_exists {
                return false;
            } else {
                game_state.player_projectile_exists = true;
            }

            let mut pos: Option<u16> = Option::None;
            if let Ok(position) = game_state
                .world
                .query_one_mut::<&Position>(game_state.player_entity)
            {
                pos = Option::Some(position.x);
            }

            if let Some(pos) = pos {
                game_state.world.spawn((
                    PlayerProjectile,
                    // We add 2 to pos, as width of player is 5 and we want projectiles to spawn in
                    // the middle
                    Position { x: pos + 2, y: 8 },
                    PrevPosition { x: pos + 2, y: 8 },
                    Velocity {
                        speed: 60.0,
                        move_accumulator: 0.0,
                        direction: Direction::None,
                    },
                    Renderable {
                        sprite_top: "⣿",
                        sprite_bottom: "",
                        width: 1,
                        destroy: false,
                        erased: false,
                    },
                ));
            }
            false
        }
        GameEvent::MovePlayerLeft => {
            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                vel.direction = Direction::Left;
            }
            false
        }
        GameEvent::MovePlayerRight => {
            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                vel.direction = Direction::Right;
            }
            false
        }
        GameEvent::MovePlayerStop => {
            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                vel.direction = Direction::None;
            }
            false
        }
        GameEvent::Pause => {
            game_state.paused = !game_state.paused;
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
                        } else if key_event.code == KeyCode::Char('w') {
                            match tx.send(GameEvent::PlayerShoot) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('p') && key_event.is_press() {
                            match tx.send(GameEvent::Pause) {
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
