use crossterm::terminal::WindowSize;

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
