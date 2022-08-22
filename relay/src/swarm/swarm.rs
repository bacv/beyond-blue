use std::sync::{Arc, Mutex};

use common::*;
use futures::{select, FutureExt, StreamExt};

use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{self, OrTransport},
        upgrade,
    },
    dns::DnsConfig,
    identity, noise,
    relay::v2::{client::Client, relay},
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::{GenTcpConfig, TcpTransport},
    Multiaddr, PeerId, Transport,
};
use log::info;
use tokio::sync::oneshot;

use crate::{Event, PeerStore, SharedStore};

type RelaySwarm = libp2p::swarm::Swarm<crate::swarm::Behaviour>;

pub struct Swarm {
    swarm: RelaySwarm,
    store: SharedStore,
    stop_tx: Option<oneshot::Sender<()>>,
    stop_rx: Option<oneshot::Receiver<()>>,
}

impl Swarm {
    pub async fn new_with_default_transport(
        local_key: identity::Keypair,
        store: Arc<Mutex<dyn PeerStore>>,
    ) -> BlueResult<Self> {
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {:?}", local_peer_id);

        let (relay_transport, _) = Client::new_transport_and_behaviour(local_peer_id);

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&local_key)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = OrTransport::new(
            relay_transport,
            DnsConfig::system(TcpTransport::new(GenTcpConfig::default().port_reuse(true)))
                .await
                .map_err(BlueError::local_err)?,
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(libp2p_yamux::YamuxConfig::default())
        .boxed();

        store
            .lock()
            .map_err(BlueError::local_err)?
            .set_relay_peer_id(&local_peer_id);

        let behaviour = crate::Behaviour::new(&local_key)?;
        Self::try_new(transport, behaviour, local_peer_id, store)
    }

    pub fn try_new(
        transport: transport::Boxed<(PeerId, StreamMuxerBox)>,
        behaviour: crate::Behaviour,
        peer_id: PeerId,
        store: Arc<Mutex<dyn PeerStore>>,
    ) -> BlueResult<Self> {
        let (stop_tx, stop_rx) = oneshot::channel();
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .dial_concurrency_factor(10_u8.try_into().map_err(BlueError::local_err)?)
            .build();
        Ok(Self {
            swarm,
            store,
            stop_tx: Some(stop_tx),
            stop_rx: Some(stop_rx),
        })
    }

    pub async fn listen_on(&mut self, addr: Multiaddr) -> BlueResult<()> {
        self.swarm.listen_on(addr).map_err(BlueError::local_err)?;
        Ok(())
    }

    pub async fn spawn(&mut self) -> BlueResult<()> {
        let rx = self
            .stop_rx
            .take()
            .ok_or_else(|| BlueError::local_err("already stopped"))?;

        select! {
            _ = rx.fuse() => {},
            _ = self.event_loop().fuse() => {},
        };

        Ok(())
    }

    pub fn stop(&mut self) {
        self.stop_tx.take();
    }

    async fn event_loop(&mut self) -> BlueResult<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(Event::Relay(relay::Event::ReservationReqAccepted {
                    src_peer_id: peer_id,
                    renewed: _,
                })) => {
                    self.store
                        .lock()
                        .map_err(BlueError::local_err)?
                        .add(peer_id);
                }
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    println!("{:?}", event)
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    self.store
                        .lock()
                        .map_err(BlueError::local_err)?
                        .append_relay_addr(address.to_string());
                    println!("Listening on {:?}", address);
                }
                _ => {}
            }
        }
    }
}
