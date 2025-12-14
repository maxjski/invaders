use std::time::Duration;

pub enum Direction {
    Right,
    Left,
    None,
}

pub struct GameState {
    pub player_updated: bool,
    pub player: PlayerShip,
}

pub struct PlayerShip {
    pub position: u16,
    pub prev_position: u16,
    pub speed: f32,
    pub move_accumulator: f32,
    pub direction: Direction,
}

impl PlayerShip {
    pub fn update(&mut self, delta_time: Duration) -> bool {
        match self.direction {
            Direction::Right => {
                self.move_accumulator += self.speed * delta_time.as_secs_f32();
            }
            Direction::Left => {
                self.move_accumulator -= self.speed * delta_time.as_secs_f32();
            }
            Direction::None => {
                self.move_accumulator = 0.0;
            }
        }

        if self.move_accumulator >= 1.0 || self.move_accumulator <= -1.0 {
            // Move in whole-cell steps, keep fractional remainder to avoid drift and asymmetry
            let steps = self.move_accumulator.trunc();
            let new_pos = self.position as i32 + steps as i32;

            if new_pos > 2 && new_pos < 114 {
                let old_pos = self.position;
                self.prev_position = old_pos;
                self.position = new_pos as u16;
                self.move_accumulator -= steps;
                return true;
            }
            self.move_accumulator -= steps;
        }
        false
    }
}
