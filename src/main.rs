use std::error::Error;
use std::io::{Stdout, Write, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::terminal::WindowSize;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, read},
    queue,
    terminal::{self, Clear, ClearType},
};

const SCREEN_WIDTH: u16 = 120;
const SCREEN_HEIGHT: u16 = 40;

struct GameState {
    wsize_updated: bool,
    player_updated: bool,
    wsize: terminal::WindowSize,
    stdout: Stdout,
    player: PlayerShip,
}

fn get_game_bounds(wsize: &WindowSize) -> (u16, u16, u16, u16) {
    let center_x = wsize.columns / 2;
    let center_y = wsize.rows / 2;
    let half_w = SCREEN_WIDTH / 2;
    let half_h = SCREEN_HEIGHT / 2;

    let left = center_x.saturating_sub(half_w);
    let right = center_x + half_w - 1; // -1 to fit inside width
    let top = center_y.saturating_sub(half_h);
    let bottom = center_y + half_h;

    (left, right, top, bottom)
}

fn render_borders(wsize: &WindowSize, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
    let center_x = wsize.columns / 2;
    let center_y = wsize.rows / 2;
    let half_w = SCREEN_WIDTH / 2;
    let half_h = SCREEN_HEIGHT / 2;

    let left = center_x.saturating_sub(half_w);
    let right = center_x + half_w - 1;
    let top = center_y.saturating_sub(half_h);
    let bottom = center_y + half_h;

    queue!(stdout, Clear(ClearType::All))?;

    // Check if terminal is too small
    if wsize.rows < SCREEN_HEIGHT + 5 || wsize.columns < SCREEN_WIDTH + 5 {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        write!(stdout, "Terminal too small")?;
        stdout.flush()?;
        return Ok(());
    }

    let horizontal_wall = "#".repeat(SCREEN_WIDTH as usize);

    // Draw Top Wall
    queue!(stdout, cursor::MoveTo(left, top))?;
    write!(stdout, "{}", horizontal_wall)?;

    // Draw Bottom Wall
    queue!(stdout, cursor::MoveTo(left, bottom))?;
    write!(stdout, "{}", horizontal_wall)?;

    queue!(stdout, cursor::MoveTo(left, bottom - 4))?;
    write!(stdout, "{}", horizontal_wall)?;

    queue!(stdout, cursor::MoveTo(left + 2, bottom - 2))?;
    write!(stdout, "q - exit")?;

    for i in 0..SCREEN_HEIGHT {
        let y = top + i;

        // Left wall
        queue!(stdout, cursor::MoveTo(left, y))?;
        write!(stdout, "#")?;

        // Right wall
        queue!(stdout, cursor::MoveTo(right, y))?;
        write!(stdout, "#")?;
    }

    stdout.flush()?;

    Ok(())
}

impl GameState {
    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if self.wsize_updated {
            self.wsize_updated = false;

            render_borders(&self.wsize, &mut self.stdout)?;
            self.player_updated = true;
        }
        if self.player_updated {
            self.player_updated = false;
            let (left, _, _, bottom) = get_game_bounds(&self.wsize);

            self.player.render_player(left, bottom, &mut self.stdout)?;
        }
        Ok(())
    }
}

enum GameEvent {
    ResizeGame,
    Tick,
    Quit,
    MovePlayerLeft,
    MovePlayerRight,
    MovePlayerStop,
}

enum Direction {
    Right,
    Left,
    None,
}

struct PlayerShip {
    hp: u32,
    position: u16,
    speed: f32,
    move_accumulator: f32,
    direction: Direction,
}

impl PlayerShip {
    fn update(&mut self, delta_time: Duration) -> bool {
        match self.direction {
            Direction::Right => {
                self.move_accumulator += self.speed * delta_time.as_secs_f32();
            }
            Direction::Left => {
                self.move_accumulator -= self.speed * delta_time.as_secs_f32();
            }
            _ => (),
        }

        if self.move_accumulator >= 1.0 || self.move_accumulator <= -1.0 {
            let new_pos = self.position as f32 + self.move_accumulator;

            if new_pos > 2.0 && new_pos < 114.0 {
                let new_pos = new_pos as u16;
                self.position = new_pos;
                return true;
            }
        }
        false
    }

    fn render_player(
        &mut self,
        left_x: u16,
        bottom_y: u16,
        stdout: &mut Stdout,
    ) -> Result<(), Box<dyn Error>> {
        queue!(
            stdout,
            cursor::MoveTo(left_x + self.position - 1, bottom_y - 7)
        )?;
        write!(stdout, " ")?;
        queue!(
            stdout,
            cursor::MoveTo(left_x + self.position + 5, bottom_y - 7)
        )?;
        write!(stdout, " ")?;

        queue!(
            stdout,
            cursor::MoveTo(left_x + self.position - 1, bottom_y - 6)
        )?;
        write!(stdout, " ")?;
        queue!(
            stdout,
            cursor::MoveTo(left_x + self.position + 5, bottom_y - 6)
        )?;
        write!(stdout, " ")?;

        queue!(stdout, cursor::MoveTo(left_x + self.position, bottom_y - 7))?;
        write!(stdout, "⣆⡜⣛⢣⣠")?;

        queue!(stdout, cursor::MoveTo(left_x + self.position, bottom_y - 6))?;
        write!(stdout, "⣿⣿⣿⣿⣿")?;

        stdout.flush()?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let tx_tick = tx.clone();
    thread::spawn(move || {
        loop {
            if tx_tick.send(GameEvent::Tick).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(16));
        }
    });

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

    let mut stdout = stdout();

    // Enter raw mode and hide cursor
    terminal::enable_raw_mode()?;
    queue!(stdout, cursor::Hide)?;

    let player = PlayerShip {
        hp: 100,
        position: 55,
        speed: 1.0,
        move_accumulator: 0.0,
        direction: Direction::None,
    };

    // Each frame is a list of lines
    let mut game_state = GameState {
        player_updated: true,
        wsize_updated: true,
        player,
        wsize: terminal::window_size()?,
        stdout,
    };

    if let Err(e) = game_state.render() {
        // We drop errors to keep and return the game_state.render() error instead
        let _ = game_state.stdout.execute(cursor::Show);
        let _ = terminal::disable_raw_mode();

        return Err(e);
    }

    let mut last_frame_time = Instant::now();

    loop {
        match rx.recv() {
            Ok(game_event) => match game_event {
                GameEvent::Quit => {
                    break;
                }
                GameEvent::ResizeGame => {
                    game_state.wsize_updated = true;
                    game_state.wsize = terminal::window_size()?;
                    match game_state.render() {
                        Ok(_) => continue,
                        Err(_) => {
                            break;
                        }
                    }
                }
                GameEvent::MovePlayerLeft => {
                    game_state.player.direction = Direction::Left;
                }
                GameEvent::MovePlayerRight => {
                    game_state.player.direction = Direction::Right;
                }
                GameEvent::MovePlayerStop => {
                    game_state.player.direction = Direction::None;
                }
                GameEvent::Tick => {
                    let now = Instant::now();
                    let dt = now.duration_since(last_frame_time);
                    last_frame_time = now;

                    game_state.player.update(dt);
                    game_state.player_updated = true;

                    match game_state.render() {
                        Ok(_) => continue,
                        Err(_) => {
                            break;
                        }
                    }
                }
            },
            Err(_) => continue,
        }
    }

    // Show cursor again and disable raw mode before exiting
    game_state.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
