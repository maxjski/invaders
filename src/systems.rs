use crate::{
    Direction, Enemy, EnemyProjectile, GameState, Player, PlayerProjectile, Position, PrevPosition,
    Render, Renderable, Velocity,
};
use crossterm::terminal;
use hecs::Entity;
use hecs::World;
use std::error::Error;
use std::io::stdout;
use std::time::Duration;

pub const SCREEN_WIDTH: u16 = 120;
pub const SCREEN_HEIGHT: u16 = 40;

pub fn create_world() -> Result<(GameState, Render), Box<dyn Error>> {
    let mut world = World::new();

    let player_entity = world.spawn((
        Player,
        Position { x: 55, y: 7 },
        PrevPosition { x: 55, y: 7 },
        Velocity {
            speed: 60.0,
            move_accumulator: 0.0,
            direction: Direction::None,
        },
        Renderable {
            sprite_top: "⣆⡜⣛⢣⣠",
            sprite_bottom: "⣿⣿⣿⣿⣿",
            width: 5,
            destroy: false,
            erased: false,
        },
    ));

    // TODO: Spawn enemies

    for x in 0..10 {
        for y in 0..3 {
            world.spawn((
                Enemy,
                Position {
                    x: 6 + x * 7,
                    y: 38 - y * 4,
                },
                PrevPosition {
                    x: 6 + x * 7,
                    y: 38 - y * 4,
                },
                Renderable {
                    sprite_top: "⢳⡴⠶⢦⡞",
                    sprite_bottom: "⠞⠫⡪⠋⠱",
                    width: 5,
                    destroy: false,
                    erased: false,
                },
                Velocity {
                    speed: 20.0,
                    move_accumulator: 0.0,
                    direction: Direction::None, // Enemy directon is stored in game state
                },
            ));
        }
    }
    // Each frame is a list of lines
    let game_state = GameState {
        world,
        player_entity,
        player_projectile_exists: false,
        enemy_direction: Direction::Right,
    };

    let stdout = stdout();

    let renderer = Render {
        stdout,
        wsize: terminal::window_size()?,
        wsize_updated: true,
    };

    Ok((game_state, renderer))
}

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

    // Queued for destruction if exists
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

    // If player projectile is assigned, we need to destory it
    if let Some(player_projectile) = player_projectile {
        world.despawn(player_projectile)?;
        game_state.player_projectile_exists = false;
    }

    // move enemies
    let mut enemies_hit_wall = false;
    for (_id, (pos, prev_pos, vel)) in world
        .query_mut::<(&mut Position, &mut PrevPosition, &mut Velocity)>()
        .with::<&Enemy>()
    {
        match game_state.enemy_direction {
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
            prev_pos.y = pos.y;

            if new_pos < 3 {
                pos.x = 2;
                enemies_hit_wall = true;
            } else if new_pos > 112 {
                pos.x = 113;
                enemies_hit_wall = true;
            } else {
                pos.x = new_pos as u16;
            }

            vel.move_accumulator -= steps;
        }
    }

    if enemies_hit_wall {
        match game_state.enemy_direction {
            Direction::Right => game_state.enemy_direction = Direction::Left,
            Direction::Left => game_state.enemy_direction = Direction::Right,
            Direction::None => game_state.enemy_direction = Direction::None,
        }

        for (_id, (pos, prev_pos)) in world
            .query_mut::<(&mut Position, &mut PrevPosition)>()
            .with::<&Enemy>()
        {
            let old_pos = pos.y;
            prev_pos.y = old_pos;
            pos.y = old_pos - 1;
        }
    }

    Ok(())

    // for (_, ()) in world.query_many_mut::<&Enemy, >
}
