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

    if wsize.rows < 30 || wsize.columns < 30 {
        stdout.execute(Clear(ClearType::All))?;

        stdout.execute(cursor::MoveTo(0, 0))?;
        write!(stdout, "Your terminal is too little dude")?;
        return Result::Ok(());
    }

    stdout.execute(Clear(ClearType::All))?;

    stdout.execute(cursor::MoveTo(0, 0))?;
    let rows = wsize.rows;
    let columns = wsize.columns;
    write!(stdout, "{rows}")?;

    stdout.execute(cursor::MoveTo(0, 1))?;
    write!(stdout, "{columns}")?;

    stdout.flush()?;

    thread::sleep(Duration::from_millis(150));

    Ok(());
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
