use super::{miner::Miner, peer_node::PeerNode};
use crate::{
    machine::Machine,
    node::{event_handle, msg_handler},
};
use futures::StreamExt;
use log::{debug, warn};
use sak_blockchain::BlockchainEvent;
use sak_p2p_ptable::{PeerStatus, PeerTable};
use std::sync::Arc;

pub(crate) struct LocalNode {
    pub(crate) peer_table: Arc<PeerTable>,
    pub(crate) machine: Arc<Machine>,
    pub(crate) miner: bool,
    pub(crate) mine_interval: Option<u64>,
}

impl LocalNode {
    pub(crate) async fn run(&self) {
        // let peer_node_rt = PeerNodeRoutine {};
        // tokio::spawn(async {
        //     peer_node_rt.run();
        // });

        let machine = self.machine.clone();
        let mine_interval = self.mine_interval.clone();
        tokio::spawn(async move {
            let miner = Miner {
                machine,
                mine_interval,
            };

            miner.run().await;
        });

        let peer_it = self.peer_table.new_iter();
        let mut peer_it_lock = peer_it.write().await;

        loop {
            let peer = match peer_it_lock.next().await {
                Ok(p) => p.clone(),
                Err(_) => continue,
            };

            let peer_node = PeerNode { peer };
            let machine = self.machine.clone();

            tokio::spawn(async move {
                run_node_routine(peer_node, machine).await;
            });
        }
    }
}

async fn run_node_routine(peer_node: PeerNode, machine: Arc<Machine>) {
    debug!(
        "Peer is registered as a peer node. Starting the routine, \
        public_key: {}",
        peer_node.peer.public_key_short()
    );

    loop {
        let mut conn = &mut peer_node.peer.transport.conn.write().await;
        // let mut bc_event_rx = machine.blockchain.bc_event_rx.write().await;
        let public_key = peer_node.peer.public_key_short();
        let mut bc_event_rx_2 = machine
            .blockchain
            .bc_event_tx
            .clone()
            .write()
            .await
            .subscribe();

        tokio::select! {
            Ok(ev) = bc_event_rx_2.recv() => {
                match ev {
                    BlockchainEvent::TxPoolStat(new_tx_hashes) => {
                        event_handle::handle_tx_pool_stat(
                            public_key,
                            &mut conn,
                            &machine,
                            new_tx_hashes,
                        ).await;
                    },
                }
            },
            maybe_msg = conn.socket.next() => {
                match maybe_msg {
                    Some(maybe_msg) => match maybe_msg {
                        Ok(msg) => {
                            let _ = msg_handler::handle_msg(
                                public_key, msg, &machine, &mut conn).await;
                        }
                        Err(err) => {
                            warn!("Failed to parse the msg, err: {}", err);
                        }
                    }
                    None => {
                        warn!("Peer has ended the connection");

                        let mut status_lock = peer_node.peer.status.write()
                            .await;

                        *status_lock = PeerStatus::Disconnected;
                        return;
                    }
                };
            }
        }
    }
}