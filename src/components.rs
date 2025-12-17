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

pub struct Player;

pub struct PlayerProjectile;

pub struct Enemy;

pub struct EnemyProjectile;
