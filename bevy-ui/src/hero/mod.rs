use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode as LyonFillMode, *};
use bevy_rapier2d::prelude::*;

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_hero.after("main_setup").label("hero"))
            .add_system(hero_force);
        //.add_system(hero_movement);
    }
}

#[derive(Component)]
pub struct Hero;

pub fn spawn_hero(mut commands: Commands) {
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
                outline_mode: StrokeMode::new(Color::YELLOW, 2.0),
            },
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(GravityScale(0.5))
        .insert(Collider::ball(10.))
        .insert(Restitution::coefficient(0.7))
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.1, 0.0),
            torque_impulse: 0.0,
        })
        .insert(Hero);
}

pub fn hero_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut position: Query<&mut Transform, With<Hero>>,
) {
    for mut transform in position.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= 2.;
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += 2.;
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= 2.;
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += 2.;
        }
    }
}

fn hero_force(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut ExternalImpulse, With<Hero>>,
) {
    for mut ext in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            ext.impulse = Vec2::new(-0.003, 0.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            ext.impulse = Vec2::new(0.003, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            ext.impulse = Vec2::new(0., -0.003);
        }
        if keyboard_input.pressed(KeyCode::W) {
            ext.impulse = Vec2::new(0., 0.003);
        }
    }
}
