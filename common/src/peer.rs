use libp2p::Multiaddr;

#[derive(Clone)]
pub struct ConnectionInfo {
    pub mutliaddr: Multiaddr,
}

#[derive(Clone)]
pub struct Peer {
    pub key: crate::Identity,
    pub conn_info: ConnectionInfo,
}
