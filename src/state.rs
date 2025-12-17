use crate::Direction;
use hecs::{Entity, World};

pub struct GameState {
    pub world: World,
    pub player_entity: Entity,
    pub player_projectile_exists: bool,
    pub enemy_direction: Direction,
}
