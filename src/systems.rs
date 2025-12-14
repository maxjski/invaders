use crate::{
    Direction, GameState, Player, PlayerProjectile, Position, PrevPosition, Renderable, Velocity,
};
use hecs::Entity;
use std::error::Error;
use std::time::Duration;

pub fn movement_system(
    delta_time: Duration,
    game_state: &mut GameState,
) -> Result<(), Box<dyn Error>> {
    let world = &mut game_state.world;
    // Move Player
    for (_id, (pos, prev_pos, vel)) in world
        .query_mut::<(&mut Position, &mut PrevPosition, &mut Velocity)>()
        .with::<&Player>()
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

            let old_pos = pos.x;
            prev_pos.x = old_pos;

            if new_pos < 2 {
                pos.x = 2;
            } else if new_pos > 113 {
                pos.x = 113;
            } else {
                pos.x = new_pos as u16;
            }

            vel.move_accumulator -= steps;
        }
    }

    // Move PlayerProjectile
    let mut player_projectile: Option<Entity> = Option::None;
    for (id, (pos, prev_pos, vel, renderable)) in world
        .query_mut::<(
            &mut Position,
            &mut PrevPosition,
            &mut Velocity,
            &mut Renderable,
        )>()
        .with::<&PlayerProjectile>()
    {
        // projectile sprite was destroyed by renderer
        if renderable.erased {
            player_projectile = Option::Some(id);
        }

        vel.move_accumulator += vel.speed * delta_time.as_secs_f32();

        if vel.move_accumulator >= 1.0 || vel.move_accumulator <= -1.0 {
            // Move in whole-cell steps, keep fractional remainder to avoid drift and asymmetry
            let steps = vel.move_accumulator.trunc();
            let new_pos = pos.y as i32 + steps as i32;

            let old_pos = pos.y;
            prev_pos.y = old_pos;

            if new_pos < 2 {
                pos.y = 2;
            } else if new_pos > 39 {
                renderable.destroy = true;
            } else {
                pos.y = new_pos as u16;
            }

            vel.move_accumulator -= steps;
        }
    }

    if let Some(player_projectile) = player_projectile {
        world.despawn(player_projectile)?;
        game_state.player_projectile_exists = false;
    }
    Ok(())
}
