use std::sync::{Arc, Mutex};

use common::*;
use futures::StreamExt;

use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{self, OrTransport},
        upgrade,
    },
    dns::DnsConfig,
    identity, noise,
    relay::v2::client::Client,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::{GenTcpConfig, TcpTransport},
    Multiaddr, PeerId, Transport,
};
use log::info;

use crate::PeerStore;

type RelaySwarm = libp2p::swarm::Swarm<crate::swarm::Behaviour>;

pub struct Swarm {
    swarm: RelaySwarm,
    store: Arc<Mutex<dyn PeerStore>>,
}

impl Swarm {
    pub async fn new_with_default_transport(
        local_key: identity::Keypair,
        store: Arc<Mutex<dyn PeerStore>>,
    ) -> BlueResult<Self> {
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {:?}", local_peer_id);

        let (relay_transport, client) = Client::new_transport_and_behaviour(local_peer_id);

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

        let behaviour = crate::Behaviour::new(&local_key)?;
        Self::try_new(transport, behaviour, local_peer_id, store)
    }

    pub fn try_new(
        transport: transport::Boxed<(PeerId, StreamMuxerBox)>,
        behaviour: crate::Behaviour,
        peer_id: PeerId,
        store: Arc<Mutex<dyn PeerStore>>,
    ) -> BlueResult<Self> {
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .dial_concurrency_factor(10_u8.try_into().map_err(BlueError::local_err)?)
            .build();
        Ok(Self { swarm, store })
    }

    pub async fn listen_on(&mut self, addr: Multiaddr) -> BlueResult<()> {
        self.swarm.listen_on(addr).map_err(BlueError::local_err)?;
        Ok(())
    }

    pub async fn spawn(&mut self) -> BlueResult<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(crate::Event::Relay(event)) => {
                    println!("{:?}", event)
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                _ => {}
            }
        }
    }
}
