use bevy::{prelude::*, utils::HashMap};

#[derive(Default)]
pub struct GameState {
    pub hero: Option<Entity>,
    pub npcs: HashMap<String, Entity>,
}
