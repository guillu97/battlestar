pub mod input;
pub mod sync;
pub mod transport;

pub use input::send_player_input;
pub use sync::{receive_game_state, update_local_ship_color, LocalShipEntity, PlayerColor};
pub use transport::setup_network;
