use std::error::Error;
use std::io::{Stdout, Write};

use crossterm::terminal::WindowSize;
use crossterm::{
    ExecutableCommand, cursor,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    queue,
    terminal::{self, Clear, ClearType},
};

use crate::{GameState, Position, PrevPosition, Renderable, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Render {
    pub wsize_updated: bool,
    pub stdout: Stdout,
    pub wsize: WindowSize,
}

impl Render {
    pub fn render(&mut self, game_state: &mut GameState) -> Result<(), Box<dyn Error>> {
        let (left, _, _, bottom) = self.get_game_bounds();

        if self.wsize.rows < SCREEN_HEIGHT + 5 || self.wsize.columns < SCREEN_WIDTH + 5 {
            queue!(self.stdout, Clear(ClearType::All))?;
            queue!(self.stdout, cursor::MoveTo(0, 0))?;
            write!(self.stdout, "Terminal too small")?;
            self.stdout.flush()?;
            return Ok(());
        }

        if self.wsize_updated {
            self.wsize_updated = false;

            self.render_borders()?;
            self.draw_menu_items(game_state.score, game_state.paused)?;
        }

        if game_state.score_updated {
            game_state.score_updated = false;
            self.draw_menu_items(game_state.score, false)?;
        }

        if game_state.paused {
            self.draw_menu_items(game_state.score, game_state.paused)?;
            return Ok(());
        }

        for (_id, (pos, prev_pos, renderable)) in
            game_state
                .world
                .query_mut::<(&Position, &PrevPosition, &mut Renderable)>()
        {
            self.draw_entity(left, bottom, pos, prev_pos, renderable)?;
            if renderable.destroy {
                renderable.erased = true;
            }
        }

        self.stdout.flush()?;

        Ok(())
    }

    fn draw_menu_items(&mut self, score: i32, paused: bool) -> Result<(), Box<dyn Error>> {
        let (left, _, _, bottom) = self.get_game_bounds();
        queue!(self.stdout, cursor::MoveTo(left + 2, bottom - 2))?;
        write!(self.stdout, "q - exit")?;

        queue!(self.stdout, cursor::MoveTo(left + 15, bottom - 2))?;
        if paused {
            write!(self.stdout, "p - unpause")?;
        } else {
            write!(self.stdout, "p - pause")?;
        }

        queue!(self.stdout, cursor::MoveTo(left + 28, bottom - 2))?;
        write!(self.stdout, "score - {}", score)?;

        Ok(())
    }

    pub fn terminal_raw_mode(&mut self) -> Result<bool, Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        let kb_flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
        let kb_enhanced = self
            .stdout
            .execute(PushKeyboardEnhancementFlags(kb_flags))
            .is_ok();
        queue!(self.stdout, cursor::Hide)?;

        Ok(kb_enhanced)
    }

    pub fn terminal_disable_raw(&mut self, kb_enhanced: bool) -> Result<(), Box<dyn Error>> {
        if kb_enhanced {
            let _ = self.stdout.execute(PopKeyboardEnhancementFlags);
        }
        self.stdout.execute(Clear(ClearType::All))?;
        self.stdout.execute(cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// It queues actions without flushing stdout
    ///
    /// Remember to flush stdout after calling
    fn draw_entity(
        &mut self,
        left: u16,
        bottom: u16,
        pos: &Position,
        prev_pos: &PrevPosition,
        renderable: &Renderable,
    ) -> Result<(), Box<dyn Error>> {
        // Erase previous sprite
        let erasor = " ".repeat(renderable.width as usize);
        queue!(
            self.stdout,
            cursor::MoveTo(left + prev_pos.x, bottom - prev_pos.y)
        )?;
        write!(self.stdout, "{}", erasor)?;
        queue!(
            self.stdout,
            cursor::MoveTo(left + prev_pos.x, bottom - prev_pos.y + 1)
        )?;
        write!(self.stdout, "{}", erasor)?;

        // Draw new sprite
        if !renderable.destroy {
            queue!(self.stdout, cursor::MoveTo(left + pos.x, bottom - pos.y))?;
            write!(self.stdout, "{}", renderable.sprite_top)?;

            queue!(
                self.stdout,
                cursor::MoveTo(left + pos.x, bottom - pos.y + 1)
            )?;
            write!(self.stdout, "{}", renderable.sprite_bottom)?;
        } else {
            queue!(self.stdout, cursor::MoveTo(left + pos.x, bottom - pos.y))?;
            write!(self.stdout, "     ")?;

            queue!(
                self.stdout,
                cursor::MoveTo(left + pos.x, bottom - pos.y + 1)
            )?;
            write!(self.stdout, "     ")?;
        }

        Ok(())
    }

    fn get_game_bounds(&self) -> (u16, u16, u16, u16) {
        let center_x = self.wsize.columns / 2;
        let center_y = self.wsize.rows / 2;
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
        let stdout = &mut self.stdout;

        let horizontal_wall = "#".repeat(SCREEN_WIDTH as usize);
        queue!(stdout, Clear(ClearType::All))?;

        // Draw Top Wall
        queue!(stdout, cursor::MoveTo(left, top))?;
        write!(stdout, "{}", horizontal_wall)?;

        // Draw Bottom Wall
        queue!(stdout, cursor::MoveTo(left, bottom))?;
        write!(stdout, "{}", horizontal_wall)?;

        queue!(stdout, cursor::MoveTo(left, bottom - 4))?;
        write!(stdout, "{}", horizontal_wall)?;

        for i in 0..SCREEN_HEIGHT {
            let y = top + i;

            // Left wall
            queue!(stdout, cursor::MoveTo(left, y))?;
            write!(stdout, "#")?;

            // Right wall
            queue!(stdout, cursor::MoveTo(right, y))?;
            write!(stdout, "#")?;
        }

        Ok(())
    }
}
