use common::BlueResult;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::ping::{Ping, PingConfig, PingEvent};
use libp2p::relay::v2::relay::{self, Relay};
use libp2p::{identity, NetworkBehaviour, PeerId};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    relay: Relay,
    ping: Ping,
    identify: Identify,
}

impl Behaviour {
    pub fn new(key: &identity::Keypair) -> BlueResult<Self> {
        let peer_id = PeerId::from(key.public());
        Ok(Self {
            relay: Relay::new(peer_id, Default::default()),
            ping: Ping::new(PingConfig::new().with_keep_alive(true)),
            identify: Identify::new(IdentifyConfig::new("/TODO/0.0.1".to_string(), key.public())),
        })
    }
}

#[derive(Debug)]
pub enum Event {
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Relay(relay::Event),
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

impl From<relay::Event> for Event {
    fn from(e: relay::Event) -> Self {
        Event::Relay(e)
    }
}
