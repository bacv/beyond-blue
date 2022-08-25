use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode as LyonFillMode, *};
use bevy_rapier2d::prelude::*;

use crate::{state::GameState, GameEvent};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_conn_events.after("hero"));
    }
}

#[derive(Component)]
pub struct Npc;

pub fn handle_conn_events(
    commands: Commands,
    game_state: ResMut<GameState>,
    from_server: Res<Arc<Mutex<mpsc::Receiver<GameEvent>>>>,
    mut query: Query<&mut Transform, With<Npc>>,
) {
    // The operation can't be blocking inside the bevy system.
    if let Ok(msg) = from_server.lock().unwrap().try_recv() {
        match msg {
            peer::NetworkEvent::NewConnection(peer_id) => {
                if game_state.npcs.get(&peer_id).is_none() {
                    spawn_npc(commands, peer_id, game_state);
                }
            }
            peer::NetworkEvent::Event(peer_id, crate::GameMessage::Move(x, y, rot)) => {
                if let Some(entity) = game_state.npcs.get(&peer_id) {
                    if let Ok(mut transform) = query.get_mut(*entity) {
                        *transform = Transform::from_xyz(x, y, 0.0);
                        transform.rotation = rot;
                    }
                }
            }
        }
    }
}

pub fn spawn_npc(mut commands: Commands, peer_id: String, mut game_state: ResMut<GameState>) {
    let shape = shapes::RegularPolygon {
        sides: 3,
        feature: shapes::RegularPolygonFeature::Radius(10.0),
        ..shapes::RegularPolygon::default()
    };

    let mut npc_entity_builder = commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: LyonFillMode::color(Color::BLACK),
            outline_mode: StrokeMode::new(Color::TEAL, 2.0),
        },
        Transform::default(),
    ));

    npc_entity_builder
        .insert(RigidBody::KinematicPositionBased)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(GravityScale(0.5))
        .insert(Collider::ball(12.))
        .insert(Restitution::coefficient(0.7))
        .insert(ExternalImpulse::default())
        .insert(Npc);

    game_state.npcs.insert(peer_id, npc_entity_builder.id());
}
