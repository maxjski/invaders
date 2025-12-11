use std::error::Error;
use std::io::{Stdout, Write, stdout};
use std::thread;
use std::time::Duration;

use crossterm::terminal::WindowSize;
use crossterm::{
    ExecutableCommand, cursor,
    terminal::{self, Clear, ClearType},
};

struct GameState {
    wsize_updated: bool,
    wsize: terminal::WindowSize,
    stdout: Stdout,
}

fn render_borders(wsize: &WindowSize, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
    if wsize.rows < 50 || wsize.columns < 50 {
        stdout.execute(Clear(ClearType::All))?;

        stdout.execute(cursor::MoveTo(0, 0))?;
        write!(stdout, "Your terminal is too little dude")?;
        return Result::Ok(());
    }

    stdout.execute(Clear(ClearType::All))?;

    for i in 0..40 {
        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20 + i, wsize.rows / 2 - 20);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 + 20, wsize.rows / 2 - 20 + i);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20, wsize.rows / 2 - 20 + i);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20 + i, wsize.rows / 2 + 20);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;
    }
    Ok(())
}

fn draw_game(game_state: &mut GameState) -> Result<(), Box<dyn Error>> {
    if game_state.wsize_updated {
        game_state.wsize_updated = false;

        render_borders(&game_state.wsize, &mut game_state.stdout)?;
    }

    let stdout = &mut game_state.stdout;
    let wsize = terminal::window_size().expect("I mean bruh, we need the window size eh?");

    if wsize.rows < 50 || wsize.columns < 50 {
        stdout.execute(Clear(ClearType::All))?;

        stdout.execute(cursor::MoveTo(0, 0))?;
        write!(stdout, "Your terminal is too little dude")?;
        return Result::Ok(());
    }

    stdout.execute(Clear(ClearType::All))?;

    for i in 0..40 {
        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20 + i, wsize.rows / 2 - 20);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 + 20, wsize.rows / 2 - 20 + i);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20, wsize.rows / 2 - 20 + i);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;

        let cursor_to_top_left = cursor::MoveTo(wsize.columns / 2 - 20 + i, wsize.rows / 2 + 20);
        stdout.execute(cursor_to_top_left)?;
        write!(stdout, "#")?;
    }

    stdout.flush()?;

    thread::sleep(Duration::from_millis(150));

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();

    // Enter raw mode and hide cursor
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;

    // Each frame is a list of lines
    let frames: Vec<Vec<&str>> = vec![
        vec!["   .-.", "  (o o)", "  | O \\", "   \\   \\", "    `~~~`"],
        vec!["   .-.", "  (o o)", "  | O )", "   /  /", "    `~~~`"],
    ];

    let mut game_state = GameState {
        wsize_updated: true,
        wsize: terminal::window_size()?,
        stdout,
    };

    for i in 0..10 {
        draw_game(&mut game_state)?;
    }

    // Show cursor again and disable raw mode before exiting
    game_state.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
