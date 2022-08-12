use crate::node::msg_handle;

use super::NodeTask;
use async_trait::async_trait;
use log::{debug, error, warn};
use sak_p2p_peertable::{Peer, PeerStatus, PeerTable};
use sak_p2p_transport::{
    handshake::{self, HandshakeInitArgs},
    Conn, Msg, TxHashSynMsg, TxSynMsg,
};
use sak_task_queue::{TaskHandler, TaskQueueError};
use sak_types::TxCandidate;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};

pub(in crate::node) struct NodeTaskHandler {
    // pub peer_table: Arc<PeerTable>,
}

#[async_trait]
impl TaskHandler<NodeTask> for NodeTaskHandler {
    async fn handle_task(&self, task: NodeTask) {
        println!("handle new task: {}", task);

        let res = match task {
            NodeTask::SendTxSyn {
                tx_candidates,
                her_public_key,
            } => {
                msg_handle::send_tx_syn(
                    tx_candidates,
                    her_public_key,
                    // &self.peer_table,
                )
                .await;
                // handle_send_tx_syn(
                //     tx_candidates,
                //     her_public_key,
                //     &self.peer_table,
                // )
                // .await
            }
            NodeTask::SendTxHashSyn {
                tx_hashes,
                her_public_key,
            } => {}
            NodeTask::SendBlockHashSyn {
                new_blocks,
                her_public_key,
            } => {}
        };

        // if let Err(err) = res {
        //     warn!("Task handle failed, err: {}", err);
        // }
    }
}

// async fn handle_send_tx_syn(
//     tx_candidates: Vec<TxCandidate>,
//     her_public_key: Option<String>,
//     peer_table: &Arc<PeerTable>,
// ) -> Result<(), TaskQueueError> {
//     if let Some(ref her_pk) = her_public_key {
//         let peer = peer_table.get_mapped_peer(&her_pk).await.ok_or(format!(
//             "peer does not exist, key: {:?}",
//             &her_public_key
//         ))?;

//         msg_handle::send_tx_syn(&peer, tx_candidates).await?;
//     } else {
//         let peer_map_lock = peer_table.get_peer_map().read().await;

//         for (_pk, peer) in peer_map_lock.iter() {
//             msg_handle::send_tx_syn(peer, tx_candidates.clone()).await?;
//         }
//     }

//     Ok(())
// }
