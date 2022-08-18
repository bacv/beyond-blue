use clap::Parser;
use common::*;
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId};
use std::error::Error;
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
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opts = Opts::parse();

    let id = peer::Identity::from_file("nothing".into());
    let (local_in, local_out) = mpsc::channel(32);
    let (remote_in, mut remote_out) = mpsc::channel(32);

    let h1 = tokio::spawn(async move {
        peer::SwarmSvc::new_with_default_transport(id.get_key())
            .await?
            .spawn(
                opts.relay_address,
                opts.remote_peer_id,
                remote_in,
                local_out,
            )
            .await;

        BlueResult::Ok(())
    });
    let remote_stream = async_stream::stream! {
        while let Some(item) = remote_out.recv().await {
            yield item;
        }
    };
    let remote_stream = remote_stream.fuse();

    tokio::pin!(remote_stream);
    tokio::join!(h1, async {
        while let msg = remote_stream.select_next_some().await {
            log::info!("Msg: {}", msg);
        }
    });

    Ok(())
}
