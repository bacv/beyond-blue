use async_std::io;
use async_std::io::prelude::BufReadExt;
use clap::Parser;
use futures::executor::block_on;
use futures::stream::StreamExt;
use libp2p::core::multiaddr::Multiaddr;
use libp2p::core::transport::OrTransport;
use libp2p::core::upgrade;
use libp2p::dns::DnsConfig;

use libp2p::noise;
use libp2p::relay::v2::client::Client;
use libp2p::tcp::{GenTcpConfig, TcpTransport};
use libp2p::Transport;
use libp2p::{identity, PeerId};
use log::info;
use std::error::Error;

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

#[derive(Debug, Parser, PartialEq)]
enum Mode {
    Dial,
    Listen,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opts = Opts::parse();

    let local_key = generate_ed25519(opts.secret_key_seed);
    let local_peer_id = PeerId::from(local_key.public());
    info!("Local peer id: {:?}", local_peer_id);

    let (relay_transport, client) = Client::new_transport_and_behaviour(local_peer_id);

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");

    let transport = OrTransport::new(
        relay_transport,
        block_on(DnsConfig::system(TcpTransport::new(
            GenTcpConfig::default().port_reuse(true),
        )))
        .unwrap(),
    )
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
    .multiplex(libp2p_yamux::YamuxConfig::default())
    .boxed();

    let behaviour = peer::Behaviour::new(client, &local_key);
    let mut swarm = peer::SwarmSvc::new(transport, behaviour, local_peer_id);

    // Wait to listen on all interfaces.
    block_on(swarm.listen());
    block_on(swarm.observe_addr());
    block_on(swarm.listen_on_relay(opts.relay_address.clone()));
    block_on(swarm.dial(opts.relay_address, opts.remote_peer_id.unwrap()));

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
    block_on(swarm.spawn_event_loop(&mut stdin));
    Ok(())
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    let secret_key = identity::ed25519::SecretKey::from_bytes(&mut bytes)
        .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
    identity::Keypair::Ed25519(secret_key.into())
}
