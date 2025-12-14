use crate::state::GameState;
use std::io::Stdout;

pub struct Render {
    stdout: Stdout,
    game_state: GameState,
}
