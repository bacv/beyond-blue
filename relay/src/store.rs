use std::collections::HashSet;

use libp2p::PeerId;

pub trait PeerStore: Send + Sync {
    fn add(&mut self, peer: PeerId);
    fn get_all(&self) -> Vec<PeerId>;
    fn remove(&mut self, peer: PeerId);
}

#[derive(Default)]
pub struct MemoryPeerStore {
    peers: HashSet<PeerId>,
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
}
