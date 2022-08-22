use std::collections::HashSet;

use libp2p::PeerId;

pub trait PeerStore: Send + Sync {
    fn add(&mut self, peer: PeerId);
    fn get_all(&self) -> Vec<PeerId>;
    fn remove(&mut self, peer: PeerId);
    fn set_relay_peer_id(&mut self, peer: &PeerId);
    fn append_relay_addr(&mut self, addr: String);
    fn get_relay(&self) -> RelayInfo;
}

#[derive(Default, Clone)]
pub struct RelayInfo {
    pub peer_id: String,
    pub addrs: Vec<String>,
}

#[derive(Default)]
pub struct MemoryPeerStore {
    /// Connection information about currently connected peers.
    peers: HashSet<PeerId>,

    /// Connection information about relay itself.
    relay: RelayInfo,
}

impl PeerStore for MemoryPeerStore {
    fn add(&mut self, peer: PeerId) {
        self.peers.insert(peer);
    }

    fn get_all(&self) -> Vec<PeerId> {
        let peers = self.peers.iter().cloned().collect::<Vec<PeerId>>();
        peers
    }

    fn remove(&mut self, peer: PeerId) {
        self.peers.remove(&peer);
    }

    fn set_relay_peer_id(&mut self, id: &PeerId) {
        self.relay = RelayInfo {
            peer_id: id.to_string(),
            ..Default::default()
        }
    }

    fn append_relay_addr(&mut self, addr: String) {
        self.relay.addrs.push(addr);
    }

    fn get_relay(&self) -> RelayInfo {
        self.relay.clone()
    }
}
