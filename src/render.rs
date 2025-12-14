use std::error::Error;
use std::io::{Stdout, Write};

use hecs::World;

use crossterm::terminal::WindowSize;
use crossterm::{
    ExecutableCommand, cursor,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    queue,
    terminal::{self, Clear, ClearType},
};

use crate::{Position, PrevPosition, Renderable};

const SCREEN_WIDTH: u16 = 120;
const SCREEN_HEIGHT: u16 = 40;

pub struct Render {
    pub wsize_updated: bool,
    pub stdout: Stdout,
    pub wsize: WindowSize,
}

impl Render {
    pub fn render(&mut self, world: &mut World) -> Result<(), Box<dyn Error>> {
        let (left, _, _, bottom) = self.get_game_bounds();

        if self.wsize_updated {
            self.wsize_updated = false;

            self.render_borders()?;
        }

        for (_id, (pos, prev_pos, renderable)) in
            world.query_mut::<(&Position, &PrevPosition, &Renderable)>()
        {
            self.draw_entity(left, bottom, pos, prev_pos, renderable)?;
        }

        self.stdout.flush()?;

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
        queue!(self.stdout, cursor::MoveTo(left + pos.x, bottom - pos.y))?;
        write!(self.stdout, "{}", renderable.sprite_top)?;

        queue!(
            self.stdout,
            cursor::MoveTo(left + pos.x, bottom - pos.y + 1)
        )?;
        write!(self.stdout, "{}", renderable.sprite_bottom)?;

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
        let wsize = &self.wsize;
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

        Ok(())
    }
}
