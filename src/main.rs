use std::error::Error;
use std::io::{Stdout, Write, stdout};
use std::thread;
use std::time::Duration;

use crossterm::{
    ExecutableCommand, cursor,
    terminal::{self, Clear, ClearType},
};

fn draw_game(stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
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

    for i in 0..10 {
        draw_game(&mut stdout);
    }

    // Show cursor again and disable raw mode before exiting
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
