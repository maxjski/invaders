use serde::{Deserialize, Serialize};

pub enum Direction {
    Right,
    Left,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct PrevPosition {
    pub x: u16,
    pub y: u16,
}

pub struct Velocity {
    pub speed: f32,
    pub move_accumulator: f32,
    pub direction: Direction,
}

pub struct Renderable {
    pub sprite_top: &'static str,
    pub sprite_bottom: &'static str,
    pub width: u16,
    pub destroy: bool,
    pub erased: bool,
}

pub struct ProjectileSpawner {
    pub probability: f64,
    pub projectile_speed: f32,
}

pub struct Player;

pub struct CoPlayer;

pub struct PlayerProjectile;

pub struct CoPlayerProjectile;

pub struct Enemy;

pub struct EnemyProjectile;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetPacket {
    PlayerInput { x: f32, shoot: bool },
    GameStateUpdate { entities: Vec<(u16, u16, u16)> },
}
