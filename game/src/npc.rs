use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode as LyonFillMode, *};
use bevy_rapier2d::prelude::*;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_npc.after("hero").label("npc"));
    }
}

#[derive(Component)]
pub struct Npc;

pub fn spawn_npc(mut commands: Commands) {
    let shape = shapes::RegularPolygon {
        sides: 3,
        feature: shapes::RegularPolygonFeature::Radius(10.0),
        ..shapes::RegularPolygon::default()
    };

    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: LyonFillMode::color(Color::BLACK),
                outline_mode: StrokeMode::new(Color::TEAL, 2.0),
            },
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(GravityScale(0.5))
        .insert(Collider::ball(10.))
        .insert(Restitution::coefficient(0.7))
        .insert(ExternalImpulse::default())
        .insert(Npc);
}
