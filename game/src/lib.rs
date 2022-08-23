mod hero;
mod npc;
mod state;

use peer::GameEvent;
use serde::{Deserialize, Serialize};

pub use hero::*;
pub use npc::*;
pub use state::*;

pub const PIXELS_PER_METER: f32 = 492.3;

#[derive(Serialize, Deserialize, Clone)]
pub enum TestGameMessage {
    Move(f32, f32),
}

pub type GameMessage = GameEvent<TestGameMessage>;
