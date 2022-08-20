use actix_web::{web, App, HttpServer};
use clap::Parser;
use common::BlueError;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use relay::{api_config, MemoryPeerStore, SharedStore};
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};

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

    let store: SharedStore = Arc::new(Mutex::new(MemoryPeerStore::default()));

    let id = common::Identity::from_file("nothing".into());
    let mut swarm = relay::Swarm::new_with_default_transport(id.get_key(), store.clone()).await?;

    let listen_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));

    let swarm = tokio::spawn(async move {
        swarm.listen_on(listen_addr).await?;
        swarm.spawn().await?;
        Ok::<(), BlueError>(())
    });

    let http_api = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone()))
            .configure(api_config)
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    _ = tokio::join!(swarm, http_api);

    Ok(())
}
