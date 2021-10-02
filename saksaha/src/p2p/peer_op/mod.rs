mod dial;
mod listen;

use super::peer::peer_store::PeerStore;
use crate::{common::SakResult, err_res};
use listen::Listen;
use std::sync::Arc;
use tokio::sync::{
    mpsc::Sender as MpscSender, oneshot::Sender as OneshotSender, Mutex,
};

pub struct PeerOp {
    peer_store: Arc<PeerStore>,
    dial_loop_tx: Arc<MpscSender<usize>>,
}

impl PeerOp {
    pub fn new(
        peer_store: Arc<PeerStore>,
        dial_loop_tx: Arc<MpscSender<usize>>,
    ) -> PeerOp {
        let peer_op = PeerOp {
            peer_store,
            dial_loop_tx,
        };

        peer_op
    }
}

impl PeerOp {
    pub async fn start(&self, peer_op_port_tx: OneshotSender<u16>) {
        let dial_loop_tx = self.dial_loop_tx.clone();

        let listen = Listen::new(dial_loop_tx);

        tokio::spawn(async move {
            listen.start_listening(peer_op_port_tx).await;
        });

        let dial = dial::Dial {};

        tokio::spawn(async move {
            match dial.start_dialing().await {
                Ok(_) => Ok(()),
                Err(err) => {
                    return err_res!(
                        "Error start peer op dialing, err: {}",
                        err
                    );
                }
            }
        });
    }
}
