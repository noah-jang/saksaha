use crate::{
    BlockHashSynMsg, BlockSynMsg, Handshake, PingMsg, TxHashSynMsg, TxSynMsg,
};

#[derive(Debug)]
pub enum Msg {
    HandshakeSyn(Handshake),

    HandshakeAck(Handshake),

    TxSyn(TxSynMsg),

    TxHashSyn(TxHashSynMsg),

    TxHashAck(TxHashSynMsg),

    BlockSyn(BlockSynMsg),

    BlockHashSyn(BlockHashSynMsg),

    BlockHashAck(BlockHashSynMsg),

    Ping(PingMsg),
}

impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Msg::HandshakeSyn(_) => write!(f, "handshake_syn"),
            Msg::HandshakeAck(_) => write!(f, "handshake_ack"),
            Msg::TxSyn(_) => write!(f, "tx_syn"),
            Msg::TxHashSyn(_) => write!(f, "tx_hash_syn"),
            Msg::TxHashAck(_) => write!(f, "tx_hash_ack"),
            Msg::BlockSyn(_) => write!(f, "block_syn"),
            Msg::BlockHashSyn(_) => write!(f, "block_hash_syn"),
            Msg::BlockHashAck(_) => write!(f, "block_hash_ack"),
            Msg::Ping(_) => write!(f, "ping"),
        }
    }
}
