use std::io::Stdout;
use crossterm::terminal::WindowSize;

pub enum Direction {
    Right,
    Left,
    None,
}

pub struct GameState {
    pub wsize_updated: bool,
    pub player_updated: bool,
    pub wsize: WindowSize,
    pub stdout: Stdout,
    pub player: PlayerShip,
}

pub struct PlayerShip {
    pub position: u16,
    pub prev_position: u16,
    pub speed: f32,
    pub move_accumulator: f32,
    pub direction: Direction,
}
