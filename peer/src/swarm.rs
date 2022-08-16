use std::net::Ipv4Addr;

use async_std::io::{self, Lines, Stdin};
use futures::stream::Fuse;
use futures::{select, FutureExt, StreamExt};
use libp2p::core::multiaddr::{Multiaddr, Protocol};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic};
use libp2p::identify::{IdentifyEvent, IdentifyInfo};
use libp2p::swarm::SwarmEvent;
use libp2p::{core::transport, swarm::SwarmBuilder, PeerId};
use libp2p_core::muxing::StreamMuxerBox;
use log::info;

use crate::Event;

type BBSwarm = libp2p::swarm::Swarm<crate::Behaviour>;

pub struct SwarmSvc {
    swarm: BBSwarm,
}

impl SwarmSvc {
    pub fn new(
        transport: transport::Boxed<(PeerId, StreamMuxerBox)>,
        behaviour: crate::Behaviour,
        peer_id: PeerId,
    ) -> Self {
        let mut swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .dial_concurrency_factor(10_u8.try_into().unwrap())
            .build();
        Self { swarm }
    }

    pub async fn listen(&mut self) {
        self.swarm
            .listen_on(
                Multiaddr::empty()
                    .with("0.0.0.0".parse::<Ipv4Addr>().unwrap().into())
                    .with(Protocol::Tcp(0)),
            )
            .unwrap();
        let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
        loop {
            futures::select! {
                event = self.swarm.next() => {
                    match event.unwrap() {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                        }
                        event => panic!("{:?}", event),
                    }
                }
                _ = delay => {
                    // Likely listening on all interfaces now, thus continuing by breaking the loop.
                    break;
                }
            }
        }
    }

    pub async fn observe_addr(&mut self) {
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match self.swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Sent { .. })) => {
                    info!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Received {
                    info: IdentifyInfo { observed_addr, .. },
                    ..
                })) => {
                    info!("Relay told us our public address: {:?}", observed_addr);
                    learned_observed_addr = true;
                }
                event => info!("{:?}", event),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }
    }

    pub async fn dial(&mut self, addr: Multiaddr, remote_peer_id: PeerId) {
        self.swarm
            .dial(
                addr.with(Protocol::P2pCircuit)
                    .with(Protocol::P2p(remote_peer_id.into())),
            )
            .unwrap();
    }

    pub async fn listen_on_relay(&mut self, relay_addr: Multiaddr) {
        self.swarm
            .listen_on(relay_addr.with(Protocol::P2pCircuit))
            .unwrap();
    }

    pub async fn spawn_event_loop(&mut self, stdin: &mut Fuse<Lines<io::BufReader<Stdin>>>) {
        loop {
            select! {
                line = stdin.select_next_some() => {
                    if let Err(e) = self.swarm
                        .behaviour_mut()
                        .gossip
                        .publish(IdentTopic::new("player-info"), line.expect("Stdin not to close").as_bytes())
                    {
                        println!("Publish error: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(Event::Relay(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Identify(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Gossipsub(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                        "Got message: {} with id: {} from peer: {:?}",
                        String::from_utf8_lossy(&message.data),
                        id,
                        peer_id
                    ),
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        info!("Established connection to {:?} via {:?}", peer_id, endpoint);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                        info!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                    }
                    _ => {}
                }
            }
        }
    }
}
