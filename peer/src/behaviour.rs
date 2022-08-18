use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use libp2p::gossipsub::{
    Gossipsub, GossipsubMessage, IdentTopic, MessageAuthenticity, MessageId, Topic, ValidationMode,
};
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::relay::v2::client::{self, Client};
use libp2p::{dcutr, gossipsub};
use libp2p::{identity, NetworkBehaviour};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    pub relay_client: Client,
    pub identify: Identify,
    pub dcutr: dcutr::behaviour::Behaviour,
    pub gossip: gossipsub::Gossipsub,

    #[behaviour(ignore)]
    pub topics: HashMap<String, IdentTopic>,
}

impl Behaviour {
    pub fn new(client: Client, key: &identity::Keypair) -> Self {
        let topics = ["player-info", "player-event"]
            .iter()
            .map(|t| (t.to_string(), Topic::new(t.to_string())))
            .collect::<HashMap<String, IdentTopic>>();

        let gossip = Self::new_gossip_config(key, topics.clone());

        Self {
            relay_client: client,
            identify: Identify::new(IdentifyConfig::new("/TODO/0.0.1".to_string(), key.public())),
            dcutr: dcutr::behaviour::Behaviour::new(),
            gossip,
            topics,
        }
    }

    fn new_gossip_config(
        key: &identity::Keypair,
        topics: HashMap<String, IdentTopic>,
    ) -> Gossipsub {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            // same content will be propagated.
            .build()
            .expect("Valid config");
        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(key.clone()), gossipsub_config)
                .expect("Correct configuration");

        // subscribes to our topics
        for (_, topic) in topics {
            gossipsub.subscribe(&topic).unwrap();
        }

        gossipsub
    }
}

#[derive(Debug)]
pub enum Event {
    Identify(IdentifyEvent),
    Relay(client::Event),
    Dcutr(dcutr::behaviour::Event),
    Gossipsub(gossipsub::GossipsubEvent),
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
