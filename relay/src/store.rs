use common::{Identity, Peer};

pub trait PeerStore: Send + Sync {
    fn add(&self, peer: Peer);
    fn get(&self, key: Identity);
    fn get_all(&self) -> Vec<Peer>;
    fn remove(&self, key: Identity);
}

#[derive(Default)]
pub struct MemoryPeerStore {
    peers: Vec<Peer>,
}

impl MemoryPeerStore {}

impl PeerStore for MemoryPeerStore {
    fn add(&self, peer: Peer) {
        todo!()
    }

    fn get(&self, key: Identity) {
        todo!()
    }

    fn get_all(&self) -> Vec<Peer> {
        todo!()
    }

    fn remove(&self, key: Identity) {
        todo!()
    }
}
