mod hero;
mod npc;
mod state;

use peer::NetworkEvent;
use serde::{Deserialize, Serialize};

pub use hero::*;
pub use npc::*;
pub use state::*;

pub const PIXELS_PER_METER: f32 = 492.3;

#[derive(Serialize, Deserialize, Clone)]
pub enum GameMessage {
    Move(f32, f32),
}

pub type GameEvent = NetworkEvent<GameMessage>;
