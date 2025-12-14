use crate::{Direction, Position, PrevPosition, Velocity};
use hecs::World;
use std::time::Duration;

pub fn movement_system(delta_time: Duration, world: &mut World) {
    for (_id, (pos, prev_pos, vel)) in
        world.query_mut::<(&mut Position, &mut PrevPosition, &mut Velocity)>()
    {
        match vel.direction {
            Direction::Right => {
                vel.move_accumulator += vel.speed * delta_time.as_secs_f32();
            }
            Direction::Left => {
                vel.move_accumulator -= vel.speed * delta_time.as_secs_f32();
            }
            Direction::None => {
                vel.move_accumulator = 0.0;
            }
        }

        if vel.move_accumulator >= 1.0 || vel.move_accumulator <= -1.0 {
            // Move in whole-cell steps, keep fractional remainder to avoid drift and asymmetry
            let steps = vel.move_accumulator.trunc();
            let new_pos = pos.x as i32 + steps as i32;

            if new_pos < 2 {
                let old_pos = pos.x;
                prev_pos.x = old_pos;
                pos.x = 2;
                vel.move_accumulator -= steps;
            } else if new_pos > 113 {
                let old_pos = pos.x;
                prev_pos.x = old_pos;
                pos.x = 113;
                vel.move_accumulator -= steps;
            } else {
                let old_pos = pos.x;
                prev_pos.x = old_pos;
                pos.x = new_pos as u16;
                vel.move_accumulator -= steps;
            }
            vel.move_accumulator -= steps;
        }
    }
}
