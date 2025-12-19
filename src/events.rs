use crate::{Direction, GameState, MainMenu, MenuItem, Player, Render, Velocity};
use std::time::Duration;

use tokio::sync::mpsc;

use crossterm::{
    event::{Event, KeyCode},
    terminal,
};

pub enum GameEvent {
    ResizeGame,
    Tick,
    Quit,
    MovePlayerLeft,
    MovePlayerLeftEnd,
    MovePlayerRight,
    MovePlayerRightEnd,
    PlayerShoot,
    PlayerShootEnd,
    Pause,
    Restart,
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
            // Handle while In Menu
            if game_state.main_menu.in_menu {
                match game_state.main_menu.active_menu_item {
                    MenuItem::HostGame => {
                        game_state.main_menu.active_menu_item = MenuItem::HostGame;
                    }
                    MenuItem::JoinGame => {
                        game_state.main_menu.active_menu_item = MenuItem::JoinGame;
                    }
                    MenuItem::PlaySolo => {
                        game_state.main_menu.in_menu = false;
                        game_state.request_clear_render = true;
                    }
                }
                return false;
            }

            game_state.player_input_handler.player_shoot = true;
            false
        }
        GameEvent::PlayerShootEnd => {
            game_state.player_input_handler.player_shoot = false;
            false
        }
        GameEvent::MovePlayerLeft => {
            // Handle when in menu
            if game_state.main_menu.in_menu {
                match game_state.main_menu.active_menu_item {
                    MenuItem::HostGame => {
                        game_state.main_menu.active_menu_item = MenuItem::PlaySolo;
                    }
                    MenuItem::JoinGame => {
                        game_state.main_menu.active_menu_item = MenuItem::HostGame;
                    }
                    MenuItem::PlaySolo => {
                        game_state.main_menu.active_menu_item = MenuItem::JoinGame;
                    }
                }
                return false;
            }

            game_state.player_input_handler.move_player_left = true;

            // TODO: Replace with direct access with Player entity stored in game_state
            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                vel.direction = Direction::Left;
            }
            false
        }
        GameEvent::MovePlayerLeftEnd => {
            game_state.player_input_handler.move_player_left = false;

            if let Ok(vel) = game_state
                .world
                .query_one_mut::<&mut Velocity>(game_state.player_entity)
            {
                if game_state.player_input_handler.move_player_right {
                    vel.direction = Direction::Right;
                } else {
                    vel.direction = Direction::None;
                }
            }
            false
        }
        GameEvent::MovePlayerRight => {
            // Handle when in menu
            if game_state.main_menu.in_menu {
                match game_state.main_menu.active_menu_item {
                    MenuItem::HostGame => {
                        game_state.main_menu.active_menu_item = MenuItem::JoinGame;
                    }
                    MenuItem::JoinGame => {
                        game_state.main_menu.active_menu_item = MenuItem::PlaySolo;
                    }
                    MenuItem::PlaySolo => {
                        game_state.main_menu.active_menu_item = MenuItem::HostGame;
                    }
                }
                return false;
            }

            game_state.player_input_handler.move_player_right = true;

            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                vel.direction = Direction::Right;
            }
            false
        }
        GameEvent::MovePlayerRightEnd => {
            game_state.player_input_handler.move_player_right = false;

            for (_, vel) in game_state
                .world
                .query_mut::<&mut Velocity>()
                .with::<&Player>()
            {
                if game_state.player_input_handler.move_player_left {
                    vel.direction = Direction::Left;
                } else {
                    vel.direction = Direction::None;
                }
            }
            false
        }
        GameEvent::Pause => {
            if !game_state.game_over {
                game_state.pause_notifier = true;
            }
            false
        }
        GameEvent::Restart => {
            game_state.restart_notifier = true;
            false
        }
        GameEvent::Tick => true,
        GameEvent::Quit => false,
    }
}

pub fn spawn_coordination_threads(tx: mpsc::UnboundedSender<GameEvent>) {
    let tx_tick = tx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        loop {
            interval.tick().await;
            if tx_tick.send(GameEvent::Tick).is_err() {
                break;
            }
        }
    });

    // handle events
    tokio::task::spawn_blocking(move || {
        loop {
            match crossterm::event::read() {
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
                        } else if key_event.code == KeyCode::Char('a') && key_event.is_release() {
                            match tx.send(GameEvent::MovePlayerLeftEnd) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('d') && key_event.is_release() {
                            match tx.send(GameEvent::MovePlayerRightEnd) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('w') && key_event.is_press() {
                            match tx.send(GameEvent::PlayerShoot) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('w') && key_event.is_release() {
                            match tx.send(GameEvent::PlayerShootEnd) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('p') && key_event.is_press() {
                            match tx.send(GameEvent::Pause) {
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        } else if key_event.code == KeyCode::Char('r') && key_event.is_press() {
                            match tx.send(GameEvent::Restart) {
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
