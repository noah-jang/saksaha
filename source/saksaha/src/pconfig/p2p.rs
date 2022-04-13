use crate::p2p::identity::Identity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedUnknownPeer {
    pub ip: String,
    pub disc_port: u16,
    pub p2p_port: Option<u16>,
    pub secret: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistedP2PConfig {
    pub identity: Identity,
    pub unknown_peers: Option<Vec<PersistedUnknownPeer>>,
    pub p2p_port: Option<u16>,
    pub disc_port: Option<u16>,
}