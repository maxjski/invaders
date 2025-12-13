use std::error::Error;
use std::io::{Stdout, Write, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    wsize: terminal::WindowSize,
    stdout: Stdout,
}

fn render_borders(wsize: &WindowSize, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
    queue!(stdout, Clear(ClearType::All))?;
    if wsize.rows < SCREEN_HEIGHT + 5 || wsize.columns < SCREEN_WIDTH + 5 {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        write!(stdout, "Your terminal is too little dude")?;
        return Result::Ok(());
    }

    for i in 0..SCREEN_HEIGHT {
        let cursor_to_top_left = cursor::MoveTo(
            wsize.columns / 2 + SCREEN_WIDTH / 2,
            wsize.rows / 2 - SCREEN_HEIGHT / 2 + i,
        );
        queue!(stdout, cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(
            wsize.columns / 2 - SCREEN_WIDTH / 2,
            wsize.rows / 2 - SCREEN_HEIGHT / 2 + i,
        );
        queue!(stdout, cursor_to_top_left)?;
        write!(stdout, "#")?;
    }

    for i in 0..SCREEN_WIDTH {
        let cursor_to_top_left = cursor::MoveTo(
            wsize.columns / 2 - SCREEN_WIDTH / 2 + i,
            wsize.rows / 2 - SCREEN_HEIGHT / 2,
        );
        queue!(stdout, cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(
            wsize.columns / 2 - SCREEN_WIDTH / 2 + i,
            wsize.rows / 2 + SCREEN_HEIGHT / 2,
        );
        queue!(stdout, cursor_to_top_left)?;
        write!(stdout, "#")?;
    }
    queue!(
        stdout,
        cursor::MoveTo(
            wsize.columns / 2 + SCREEN_WIDTH / 2,
            wsize.rows / 2 + SCREEN_HEIGHT / 2,
        )
    )?;
    write!(stdout, "#")?;
    stdout.flush()?;

    Ok(())
}

impl GameState {
    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if self.wsize_updated {
            self.wsize_updated = false;

            render_borders(&self.wsize, &mut self.stdout)?;
        }
        Ok(())
    }
}

enum GameEvent {
    Input(KeyEvent),
    ResizeGame,
    Tick,
    Quit,
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

    // Each frame is a list of lines
    let mut game_state = GameState {
        wsize_updated: true,
        wsize: terminal::window_size()?,
        stdout,
    };

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
                _ => continue,
            },
            Err(_) => continue,
        }
    }

    game_state.render()?;

    // Show cursor again and disable raw mode before exiting
    game_state.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
