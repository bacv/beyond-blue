use libp2p::identify::{Identify, IdentifyEvent};
use libp2p::ping::{Ping, PingEvent};
use libp2p::relay::v2::client::{self, Client};
use libp2p::NetworkBehaviour;
use libp2p::{dcutr, gossipsub};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    pub relay_client: Client,
    pub ping: Ping,
    pub identify: Identify,
    pub dcutr: dcutr::behaviour::Behaviour,
    pub gossip: gossipsub::Gossipsub,
}

#[derive(Debug)]
pub enum Event {
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Relay(client::Event),
    Dcutr(dcutr::behaviour::Event),
    Gossipsub(gossipsub::GossipsubEvent),
}

impl From<PingEvent> for Event {
    fn from(e: PingEvent) -> Self {
        Event::Ping(e)
    }
}

impl From<IdentifyEvent> for Event {
    fn from(e: IdentifyEvent) -> Self {
        Event::Identify(e)
    }
}

impl From<client::Event> for Event {
    fn from(e: client::Event) -> Self {
        Event::Relay(e)
    }
}

impl From<dcutr::behaviour::Event> for Event {
    fn from(e: dcutr::behaviour::Event) -> Self {
        Event::Dcutr(e)
    }
}

impl From<gossipsub::GossipsubEvent> for Event {
    fn from(e: gossipsub::GossipsubEvent) -> Self {
        Event::Gossipsub(e)
    }
}
