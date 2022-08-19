use clap::Parser;
use futures::executor::block_on;
use futures::stream::StreamExt;
use libp2p::core::upgrade;
use libp2p::identify::{Identify, IdentifyConfig};
use libp2p::multiaddr::Protocol;
use libp2p::ping::{Ping, PingConfig};
use libp2p::relay::v2::relay::Relay;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::tcp::TcpTransport;
use libp2p::Transport;
use libp2p::{identity, PeerId};
use libp2p::{noise, Multiaddr};
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Parser)]
#[clap(name = "libp2p relay")]
struct Opt {
    /// Determine if the relay listen on ipv6 or ipv4 loopback address. the default is ipv4
    #[clap(long)]
    use_ipv6: Option<bool>,

    /// Fixed value to generate deterministic peer id
    #[clap(long)]
    secret_key_seed: u8,

    /// The port used to listen on all interfaces
    #[clap(long)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let opt = Opt::parse();

    let id = common::Identity::from_file("nothing".into());
    let mut swarm = relay::Swarm::new_with_default_transport(id.get_key()).await?;
    // Listen on all interfaces
    let listen_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));
    swarm.listen_on(listen_addr).await?;
    swarm.spawn().await;
    Ok(())
}
