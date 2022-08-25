use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use beyond_blue::{GameMessage, GameState, HeroAction, HeroPlugin, NpcPlugin, PIXELS_PER_METER};
use clap::Parser;
use common::*;
use leafwing_input_manager::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

const WINDOW_WIDTH: usize = 600;
const WINDOW_HEIGHT: usize = 480;

#[derive(Debug, Parser)]
#[clap(name = "Example Beyond Blue peer")]
struct Opts {
    /// The listening address
    #[clap(long)]
    relay_address: url::Url,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Beyond Blue".to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            ..Default::default()
        })
        .insert_resource(Msaa::default())
        .insert_resource(GameState::default())
        .insert_resource(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .insert_resource(opts)
        .add_plugins(DefaultPlugins)
        .add_plugin(HeroPlugin)
        .add_plugin(NpcPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(InputManagerPlugin::<HeroAction>::default())
        .add_startup_system(setup_physics.label("main_setup"))
        .add_startup_system(setup_network.label("net_setup"))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .run();
}

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::new(0., 0.);
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: 1.,
        substeps: 30,
    };
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_network(mut commands: Commands, runtime: Res<Runtime>, opts: Res<Opts>) {
    let (local_in, local_out) = mpsc::channel(32);
    let (remote_in, remote_out) = mpsc::channel(32);

    let relay_address = opts.relay_address.clone();
    runtime.spawn(async move {
        let id = common::Identity::from_file("nothing".into());

        tokio::spawn(async move {
            let res = peer::Swarm::new_with_default_transport(id.get_key())
                .await?
                .spawn::<GameMessage>(relay_address, remote_in, local_out)
                .await;

            log::info!("Game swarm result: {:?}", res);

            BlueResult::Ok(())
        });
    });

    commands.insert_resource(local_in);
    commands.insert_resource(Arc::new(Mutex::new(remote_out)));
}
