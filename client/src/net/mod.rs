pub mod input;
pub mod sync;
pub mod transport;

pub use input::{gather_player_input, send_player_input, InputThrottle, PlayerInput};
pub use sync::{receive_game_state, update_local_ship_color, LocalShipEntity, PlayerColor};
pub use transport::{poll_connection_state, setup_network};
