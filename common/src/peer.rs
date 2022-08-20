use libp2p::Multiaddr;

pub struct ConnectionInfo {
    pub mutliaddr: Multiaddr,
}

pub struct Peer {
    pub key: crate::Identity,
    pub conn_info: ConnectionInfo,
}
