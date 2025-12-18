use crate::{
    Direction, Enemy, EnemyProjectile, GameState, MainMenu, MenuItem, Player, PlayerInputHandler,
    PlayerProjectile, Position, PrevPosition, ProjectileSpawner, Render, Renderable, Velocity,
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

    // Each frame is a list of lines
    let mut game_state = GameState {
        world,
        player_lives: 3,
        player_entity,
        player_projectile_exists: false,
        enemy_direction: Direction::Right,
        score_updated: true,
        score: 0,
        high_score: 0,
        enemy_speed_multiplier: 1.0,
        enemy_proj_prob_multiplier: 1.0,
        enemy_amount: 30,
        game_over: false,
        game_over_notifier: false,
        paused: false,
        pause_notifier: false,
        restart_notifier: false,
        player_input_handler: PlayerInputHandler {
            player_shoot: false,
            move_player_right: false,
            move_player_left: false,
        },
        main_menu: MainMenu {
            in_menu: true,
            active_menu_item: MenuItem::HostGame,
        },
    };

    spawn_enemies(
        game_state.enemy_proj_prob_multiplier,
        game_state.enemy_speed_multiplier,
        &mut game_state.world,
    );
    let stdout = stdout();

    let renderer = Render {
        stdout,
        wsize: terminal::window_size()?,
        wsize_updated: true,
    };

    Ok((game_state, renderer))
}

pub fn restart_world(high_score: i32) -> Result<(GameState, Render), Box<dyn Error>> {
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

    // Each frame is a list of lines
    let mut game_state = GameState {
        world,
        player_lives: 3,
        player_entity,
        player_projectile_exists: false,
        enemy_direction: Direction::Right,
        score_updated: true,
        score: 0,
        high_score,
        enemy_speed_multiplier: 1.0,
        enemy_proj_prob_multiplier: 1.0,
        enemy_amount: 30,
        game_over: false,
        game_over_notifier: false,
        paused: false,
        pause_notifier: false,
        restart_notifier: false,
        player_input_handler: PlayerInputHandler {
            player_shoot: false,
            move_player_right: false,
            move_player_left: false,
        },
        main_menu: MainMenu {
            in_menu: false,
            active_menu_item: MenuItem::PlaySolo,
        },
    };
    spawn_enemies(
        game_state.enemy_proj_prob_multiplier,
        game_state.enemy_speed_multiplier,
        &mut game_state.world,
    );

    let stdout = stdout();

    let renderer = Render {
        stdout,
        wsize: terminal::window_size()?,
        wsize_updated: true,
    };

    Ok((game_state, renderer))
}

fn spawn_enemies(proj_multiplier: f32, speed_multiplier: f32, world: &mut World) {
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
                    speed: 20.0 * speed_multiplier,
                    move_accumulator: 0.0,
                    direction: Direction::None, // Enemy directon is stored in game state
                },
                ProjectileSpawner {
                    probability: 0.1 * proj_multiplier as f64,
                    projectile_speed: -20.0,
                },
            ));
        }
    }
}

pub fn process_tick(
    delta_time: Duration,
    game_state: &mut GameState,
) -> Result<(), Box<dyn Error>> {
    move_player(delta_time, &mut game_state.world);
    process_player_projectile(delta_time, game_state)?;

    process_enemies(delta_time, game_state);
    enemy_collision_detection(game_state);

    process_enemy_projectiles(delta_time, game_state)?;
    player_collision_detection(game_state);

    entity_cleanup(&mut game_state.world)?;

    Ok(())
}

fn spawn_player_projectile(game_state: &mut GameState) {
    if game_state.player_projectile_exists || !game_state.player_input_handler.player_shoot {
        return;
    } else {
        game_state.player_projectile_exists = true;
    }

    let mut pos: Option<u16> = Option::None;
    if let Ok(position) = game_state
        .world
        .query_one_mut::<&Position>(game_state.player_entity)
    {
        pos = Option::Some(position.x);
    }

    if let Some(pos) = pos {
        game_state.world.spawn((
            PlayerProjectile,
            // We add 2 to pos, as width of player is 5 and we want projectiles to spawn in
            // the middle
            Position { x: pos + 2, y: 8 },
            PrevPosition { x: pos + 2, y: 8 },
            Velocity {
                speed: 60.0,
                move_accumulator: 0.0,
                direction: Direction::None,
            },
            Renderable {
                sprite_top: "⣿",
                sprite_bottom: "",
                width: 1,
                destroy: false,
                erased: false,
            },
        ));
    }
}

fn process_player_projectile(
    delta_time: Duration,
    game_state: &mut GameState,
) -> Result<(), Box<dyn Error>> {
    spawn_player_projectile(game_state);

    let mut player_projectile: Option<Entity> = Option::None;
    for (id, (pos, prev_pos, vel, renderable)) in game_state
        .world
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
        game_state.world.despawn(player_projectile)?;
        game_state.player_projectile_exists = false;
    }

    Ok(())
}

fn move_player(delta_time: Duration, world: &mut World) {
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
}

fn process_enemies(delta_time: Duration, game_state: &mut GameState) {
    let mut enemies_hit_wall = false;
    let mut projectiles_to_spawn: Vec<(Position, Velocity)> = Vec::new();

    for (_id, (pos, prev_pos, vel, proj_spawn)) in game_state
        .world
        .query_mut::<(
            &mut Position,
            &mut PrevPosition,
            &mut Velocity,
            &ProjectileSpawner,
        )>()
        .with::<&Enemy>()
    {
        let chance = rand::random::<f64>() * 100.0;

        if proj_spawn.probability > chance {
            projectiles_to_spawn.push((
                Position {
                    x: pos.x + 2,
                    y: pos.y - 1,
                },
                Velocity {
                    move_accumulator: 0.0,
                    speed: proj_spawn.projectile_speed,
                    direction: Direction::None,
                },
            ))
        }

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

    for (pos, vel) in projectiles_to_spawn {
        game_state.world.spawn((
            EnemyProjectile,
            pos,
            vel,
            PrevPosition { x: pos.x, y: pos.y },
            Renderable {
                sprite_top: "",
                sprite_bottom: "⣿",
                width: 1,
                destroy: false,
                erased: false,
            },
        ));
    }

    // Switch enemy direction when wall is hit
    if enemies_hit_wall {
        match game_state.enemy_direction {
            Direction::Right => game_state.enemy_direction = Direction::Left,
            Direction::Left => game_state.enemy_direction = Direction::Right,
            Direction::None => game_state.enemy_direction = Direction::None,
        }

        for (_id, (pos, prev_pos)) in game_state
            .world
            .query_mut::<(&mut Position, &mut PrevPosition)>()
            .with::<&Enemy>()
        {
            let old_pos = pos.y;
            prev_pos.y = old_pos;
            pos.y = old_pos - 1;
            if pos.y <= 10 {
                // Enemies flew too low
                game_state.game_over_notifier = true;
            }
        }
    }
}

fn process_enemy_projectiles(
    delta_time: Duration,
    game_state: &mut GameState,
) -> Result<(), Box<dyn Error>> {
    let mut projectiles_to_erase: Vec<Entity> = Vec::new();

    for (id, (pos, prev_pos, vel, renderable)) in game_state
        .world
        .query_mut::<(
            &mut Position,
            &mut PrevPosition,
            &mut Velocity,
            &mut Renderable,
        )>()
        .with::<&EnemyProjectile>()
    {
        // projectile sprite was destroyed by renderer
        if renderable.erased {
            projectiles_to_erase.push(id);
        }

        vel.move_accumulator += vel.speed * delta_time.as_secs_f32();

        if vel.move_accumulator >= 1.0 || vel.move_accumulator <= -1.0 {
            // Move in whole-cell steps, keep fractional remainder to avoid drift and asymmetry
            let steps = vel.move_accumulator.trunc();
            let new_pos = pos.y as i32 + steps as i32;

            let old_pos = pos.y;
            prev_pos.y = old_pos;

            if new_pos < 6 || new_pos > 39 {
                renderable.destroy = true;
            } else {
                pos.y = new_pos as u16;
            }

            vel.move_accumulator -= steps;
        }
    }

    for proj in projectiles_to_erase {
        game_state.world.despawn(proj)?;
    }

    Ok(())
}

fn entity_cleanup(world: &mut World) -> Result<(), Box<dyn Error>> {
    let mut entities_erased: Vec<Entity> = Vec::new();

    for (id, renderable) in world.query_mut::<&Renderable>() {
        if renderable.erased {
            entities_erased.push(id);
        }
    }

    for entity_id in entities_erased {
        world.despawn(entity_id)?;
    }

    Ok(())
}

// TODO: Collision system is quite bad:
// Instead it should detect any objects with Positions and we need a new
// property like 'Collides'. We should create a map of collisions that occured
// with defined behaviors. Collides could be an enum struct with Collides::LoseHP or
// Collides::DestroySelf etc
fn enemy_collision_detection(game_state: &mut GameState) {
    let mut entities_hit: Vec<Entity> = Vec::new();
    let mut need_new_enemies = false;

    {
        let projectile_data = game_state
            .world
            .query::<&Position>()
            .with::<&PlayerProjectile>()
            .iter()
            .map(|(id, pos)| (id, *pos))
            .next();

        if let Some((proj_id, proj_pos)) = projectile_data {
            for (enemy_id, (enemy_pos, renderable)) in game_state
                .world
                .query_mut::<(&Position, &Renderable)>()
                .with::<&Enemy>()
            {
                if proj_pos.x >= enemy_pos.x
                    && proj_pos.x <= enemy_pos.x + renderable.width
                    && proj_pos.y == enemy_pos.y
                {
                    entities_hit.push(proj_id);
                    entities_hit.push(enemy_id);
                    game_state.score += 10;
                    game_state.score_updated = true;
                    if game_state.enemy_amount == 1 {
                        game_state.enemy_speed_multiplier *= 1.2;
                        game_state.enemy_proj_prob_multiplier *= 3.0;
                        game_state.enemy_amount = 30;
                        need_new_enemies = true;
                    } else {
                        game_state.enemy_amount -= 1;
                    }
                }
            }
        }
    }

    if need_new_enemies {
        spawn_enemies(
            game_state.enemy_proj_prob_multiplier,
            game_state.enemy_speed_multiplier,
            &mut game_state.world,
        );
    }

    for entity_id in entities_hit {
        if let Ok(mut renderable) = game_state.world.get::<&mut Renderable>(entity_id) {
            renderable.destroy = true;
        }
    }
}

fn player_collision_detection(game_state: &mut GameState) {
    let mut player_hit = false;

    let player_data = game_state
        .world
        .query::<&Position>()
        .with::<&Player>()
        .iter()
        .map(|(id, pos)| (id, *pos))
        .next();

    if let Some((_, player_pos)) = player_data {
        for (_, (proj_pos, renderable)) in game_state
            .world
            .query_mut::<(&Position, &mut Renderable)>()
            .with::<&EnemyProjectile>()
        {
            if proj_pos.x >= player_pos.x
                && proj_pos.x <= player_pos.x + 5
                && player_pos.y == proj_pos.y - 1
            {
                player_hit = true;
                renderable.destroy = true;
            }
        }
    }

    if player_hit {
        game_state.player_lives -= 1;
        game_state.score_updated = true;
        if game_state.player_lives == 0 {
            game_state.game_over_notifier = true;
        }
    }
}
