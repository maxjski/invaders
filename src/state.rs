use crate::Direction;
use hecs::{Entity, World};

pub struct GameState {
    pub world: World,

    pub player_lives: u16,
    pub player_entity: Entity,
    pub player_projectile_exists: bool,
    pub enemy_direction: Direction,
    pub score_updated: bool,
    pub score: i32,
    pub high_score: i32,

    pub enemy_speed_multiplier: f32,
    pub enemy_proj_prob_multiplier: f32,
    pub enemy_amount: u16,

    pub game_over: bool,
    pub game_over_notifier: bool,
    pub paused: bool,
    pub pause_notifier: bool,

    pub restart_notifier: bool,

    pub player_input_handler: PlayerInputHandler,
    pub main_menu: MainMenu,
    pub networking: GameNetworking,
    pub request_clear_render: bool,
}

pub struct PlayerInputHandler {
    pub player_shoot: bool,
    pub move_player_left: bool,
    pub move_player_right: bool,
}

pub enum MenuItem {
    HostGame,
    JoinGame,
    PlaySolo,
}

pub struct MainMenu {
    pub in_menu: bool,
    pub active_menu_item: MenuItem,

    pub hosting: bool,
}

pub struct GameNetworking {
    pub is_listening: bool,
    pub host: bool,
    pub peer: Option<std::net::SocketAddr>,
}
