use crate::state::GameState;
use crossterm::terminal::WindowSize;
use std::error::Error;
use std::io::{Stdout, Write};

use crossterm::{
    ExecutableCommand, cursor,
    event::{
        Event, KeyCode, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags, read,
    },
    queue,
    terminal::{self, Clear, ClearType},
};
const SCREEN_WIDTH: u16 = 120;
const SCREEN_HEIGHT: u16 = 40;

pub struct Render {
    pub stdout: Stdout,
    pub game_state: GameState,
}

impl Render {
    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if self.game_state.wsize_updated {
            self.game_state.wsize_updated = false;

            self.render_borders()?;
            self.game_state.player_updated = true;
        }
        if self.game_state.player_updated {
            self.game_state.player_updated = false;
            let (left, _, _, bottom) = self.get_game_bounds();

            self.game_state
                .player
                .render_player(left, bottom, &mut self.stdout)?;
        }
        Ok(())
    }

    fn get_game_bounds(&self) -> (u16, u16, u16, u16) {
        let center_x = self.game_state.wsize.columns / 2;
        let center_y = self.game_state.wsize.rows / 2;
        let half_w = SCREEN_WIDTH / 2;
        let half_h = SCREEN_HEIGHT / 2;

        let left = center_x.saturating_sub(half_w);
        let right = center_x + half_w - 1; // -1 to fit inside width
        let top = center_y.saturating_sub(half_h);
        let bottom = center_y + half_h;

        (left, right, top, bottom)
    }

    fn render_borders(&mut self) -> Result<(), Box<dyn Error>> {
        let (left, right, top, bottom) = self.get_game_bounds();
        let wsize = &self.game_state.wsize;
        let stdout = &mut self.stdout;

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
}
