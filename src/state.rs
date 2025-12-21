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

pub enum Screen {
    Main,
    Hosting,
    Joining,
    Game,
}

pub struct MainMenu {
    pub active_menu_item: MenuItem,
    pub screen: Screen,
}

pub struct GameNetworking {
    pub stay_online: bool,
    pub listener_task: Option<tokio::task::JoinHandle<()>>,
    pub host: bool,
    pub peer: Option<std::net::SocketAddr>,
}

impl GameState {
    pub fn exit_to_menu(&mut self) {
        self.main_menu.screen = Screen::Main;
        self.request_clear_render = true;
        self.restart_notifier = true;
        self.networking.reset();
    }
}

impl GameNetworking {
    pub fn host(&mut self) {
        self.stay_online = true;
    }

    pub fn reset(&mut self) {
        self.stay_online = false;
        self.host = false;
        self.listener_task = Option::None;
        self.peer = Option::None;
    }
}
