use std::time::Duration;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode as LyonFillMode, *};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use tokio::sync::mpsc;

use crate::{GameMessage, GameState};

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_hero.after("net_setup").label("hero"))
            .add_system(hero_force.after("hero"))
            .add_system(hero_dampening_system.after("hero"));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum HeroAction {
    Forward,
    RotateLeft,
    RotateRight,
}

#[derive(Component)]
pub struct Hero {
    rotation_speed: f32,
    thrust: f32,
    update_timer: Timer,
}

pub fn spawn_hero(mut commands: Commands, mut game_state: ResMut<GameState>) {
    let input_map = InputMap::new([
        (KeyCode::W, HeroAction::Forward),
        (KeyCode::Up, HeroAction::Forward),
        (KeyCode::A, HeroAction::RotateLeft),
        (KeyCode::Left, HeroAction::RotateLeft),
        (KeyCode::D, HeroAction::RotateRight),
        (KeyCode::Right, HeroAction::RotateRight),
    ]);

    let shape = shapes::RegularPolygon {
        sides: 3,
        feature: shapes::RegularPolygonFeature::Radius(10.0),
        ..shapes::RegularPolygon::default()
    };

    let mut hero_entity_builder = commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: LyonFillMode::color(Color::BLACK),
            outline_mode: StrokeMode::new(Color::YELLOW, 2.0),
        },
        Transform::default(),
    ));

    hero_entity_builder
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(GravityScale(0.5))
        .insert(Collider::ball(8.))
        .insert(Restitution::coefficient(0.7))
        .insert(ExternalImpulse::default())
        .insert(Velocity::linear(Vec2::ZERO))
        .insert_bundle(InputManagerBundle::<HeroAction> {
            action_state: ActionState::default(),
            input_map,
        })
        .insert(Hero {
            rotation_speed: 3.0,
            thrust: 6.0,
            update_timer: Timer::new(Duration::from_millis(40), true),
        });

    game_state.hero = Some(hero_entity_builder.id());
}

pub fn hero_dampening_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<&mut Velocity>,
) {
    if let Ok(mut velocity) = query.get_component_mut::<Velocity>(game_state.hero.unwrap()) {
        let elapsed = time.delta_seconds();
        velocity.angvel *= 0.1f32.powf(elapsed);
        velocity.linvel *= 0.4f32.powf(elapsed);
    }
}

fn hero_force(
    to_server: ResMut<mpsc::Sender<GameMessage>>,
    game_state: ResMut<GameState>,
    action_state_query: Query<&ActionState<HeroAction>>,
    mut query: Query<(&mut ExternalImpulse, &mut Velocity, &Transform, &mut Hero)>,
    time: Res<Time>,
) {
    if let Some(hero) = game_state.hero {
        if let Ok(action_state) = action_state_query.get(hero) {
            let thrust = if action_state.pressed(HeroAction::Forward) {
                0.00001
            } else {
                0.0
            };
            let rotation = if action_state.pressed(HeroAction::RotateLeft) {
                0.01
            } else if action_state.pressed(HeroAction::RotateRight) {
                -0.01
            } else {
                0.0
            };
            if let Ok((mut impulse, mut velocity, transform, hero)) = query.get_mut(hero) {
                if rotation != 0.0 {
                    velocity.angvel = rotation as f32 * hero.rotation_speed;
                }
                impulse.impulse =
                    (transform.rotation * (Vec3::Y * thrust * hero.thrust)).truncate();
            }
        }

        if let Ok((_, _, transform, mut hero)) = query.get_mut(hero) {
            hero.update_timer.tick(time.delta());
            if hero.update_timer.finished() {
                _ = to_server.try_send(GameMessage::Move(
                    transform.translation.x,
                    transform.translation.y,
                    transform.rotation,
                ));
            }
        }
    }
}
