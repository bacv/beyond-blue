use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use beyond_blue::{HeroPlugin, NpcPlugin, PIXELS_PER_METER};
use clap::Parser;
use common::*;
use libp2p::{Multiaddr, PeerId};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[derive(Debug, Parser)]
#[clap(name = "Example Beyond Blue peer")]
struct Opts {
    /// Fixed value to generate deterministic peer id.
    #[clap(long)]
    secret_key_seed: u8,

    /// The listening address
    #[clap(long)]
    relay_address: Multiaddr,

    /// Peer ID of the remote peer to hole punch to.
    #[clap(long)]
    remote_peer_id: Option<PeerId>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .add_plugins(DefaultPlugins)
        .add_plugin(HeroPlugin)
        .add_plugin(NpcPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_physics.label("main_setup"))
        .add_startup_system(setup_network)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .run();
}

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::new(0., 0.);
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_network(mut commands: Commands, runtime: Res<Runtime>) {
    let (local_in, local_out) = mpsc::channel(32);
    let (remote_in, mut remote_out) = mpsc::channel(32);

    runtime.spawn(async move {
        let id = common::Identity::from_file("nothing".into());
        let relay_address =
            "/ip4/145.239.92.79/tcp/8842/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";

        tokio::spawn(async move {
            peer::SwarmSvc::new_with_default_transport(id.get_key())
                .await?
                .spawn(
                    relay_address.try_into().map_err(BlueError::local_err)?,
                    None,
                    remote_in,
                    local_out,
                )
                .await;

            BlueResult::Ok(())
        });
    });
    commands.insert_resource(local_in);
}
