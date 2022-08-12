use crate::{
    machine::Machine,
    node::{task::NodeTask, SaksahaNodeError},
    SaksahaError,
};
use futures::{stream::SplitSink, SinkExt};
use log::{debug, info, warn};
use sak_p2p_peertable::{Peer, PeerTable};
use sak_p2p_transport::{
    BlockHashSynMsg, BlockSynMsg, Msg, MsgType, SendReceipt, TxAckMsg,
    TxHashSynMsg, TxSynMsg, UpgradedConn, UpgradedP2PCodec,
};
use sak_task_queue::TaskQueue;
use sak_types::TxCandidate;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLockWriteGuard};

pub(in crate::node) async fn send_tx_syn(
    tx_candidates: Vec<TxCandidate>,
    her_public_key: Option<String>,
    // peer_table: &Arc<PeerTable>,
    // peer: &Arc<Peer>,
    // tx_candidates: Vec<TxCandidate>,
) -> Result<(), SaksahaNodeError> {
    // if let Some(ref her_pk) = her_public_key {
    //     let peer = peer_table.get_mapped_peer(&her_pk).await.ok_or(format!(
    //         "peer does not exist, key: {:?}",
    //         &her_public_key
    //     ))?;

    //     send_tx_syn_each(&peer, tx_candidates).await?;
    // } else {
    //     let peer_map_lock = peer_table.get_peer_map().read().await;

    //     for (_pk, peer) in peer_map_lock.iter() {
    //         send_tx_syn_each(peer, tx_candidates.clone()).await?;
    //     }
    // }

    Ok(())
}

pub(in crate::node) async fn recv_tx_syn(
    tx_syn: TxSynMsg,
    machine: &Machine,
    mut conn: RwLockWriteGuard<'_, UpgradedConn>,
) -> Result<SendReceipt, SaksahaNodeError> {
    machine
        .blockchain
        .dist_ledger
        .apis
        .insert_into_pool(tx_syn.tx_candidates)
        .await;

    let tx_ack_msg = Msg::TxAck(TxAckMsg {});

    let receipt = conn.send(tx_ack_msg).await?;

    Ok(receipt)
}

async fn send_tx_syn_each(
    peer: &Arc<Peer>,
    tx_candidates: Vec<TxCandidate>,
) -> Result<(), SaksahaNodeError> {
    let mut conn = peer.get_transport().conn.write().await;

    let tx_syn_msg = Msg::TxSyn(TxSynMsg { tx_candidates });

    conn.send(tx_syn_msg).await?;

    let msg = conn
        .next_msg()
        .await
        .ok_or(format!("tx syn needs to be followed by tx syn ack"))??;

    let msg = match msg {
        Msg::TxAck(m) => (),
        _ => {
            return Err(
                format!("Only tx ack should arrive at this point").into()
            );
        }
    };

    Ok(())
}

pub(super) async fn handle_tx_hash_syn<'a>(
    tx_hash_syn_msg: TxHashSynMsg,
    machine: &Machine,
    conn: &'a mut RwLockWriteGuard<'_, UpgradedConn>,
    task_queue: &Arc<TaskQueue<NodeTask>>,
    peer: &Arc<Peer>,
) -> Result<(), SaksahaNodeError> {
    let txs_to_request = machine
        .blockchain
        .dist_ledger
        .apis
        .get_tx_pool_diff(tx_hash_syn_msg.tx_hashes)
        .await;

    match conn
        .send(Msg::TxHashAck(TxHashSynMsg {
            tx_hashes: txs_to_request,
        }))
        .await
    {
        Ok(_) => {}
        Err(err) => {
            warn!("Failed to handle TxHashSyn msg, err: {}", err,);
        }
    };

    let msg = conn
        .next_msg()
        .await
        .ok_or(format!("tx hash ack is empty"))??;

    let tx_hash_ack = match msg {
        Msg::TxHashAck(m) => m,
        _ => {
            return Err(format!(
                "Only tx hash msg is expected, msg: {:?}",
                msg
            )
            .into());
        }
    };

    let tx_candidates = machine
        .blockchain
        .dist_ledger
        .apis
        .get_txs_from_pool(tx_hash_ack.tx_hashes)
        .await;

    task_queue
        .push_back(NodeTask::SendTxSyn {
            tx_candidates,
            her_public_key: Some(peer.get_public_key().to_string()),
        })
        .await?;

    // if !tx_candidates.is_empty() {
    //     match conn.send(Msg::TxSyn(TxSynMsg { tx_candidates })).await {
    //         Ok(_) => {
    //             // info!("Sending TxSyn, public_key: {}", public_key);
    //         }
    //         Err(err) => {
    //             info!("Failed to handle TxHashAck, err: {}", err,);
    //         }
    //     }
    // }

    Ok(())
}

// pub(super) async fn handle_tx_hash_ack<'a>(
//     tx_hash_ack: TxHashSynMsg,
//     machine: &Machine,
//     conn: &'a mut RwLockWriteGuard<'_, UpgradedConn>,
// ) {
//     let tx_candidates = machine
//         .blockchain
//         .dist_ledger
//         .apis
//         .get_txs_from_pool(tx_hash_ack.tx_hashes)
//         .await;

//     if !tx_candidates.is_empty() {
//         match conn.send(Msg::TxSyn(TxSynMsg { tx_candidates })).await {
//             Ok(_) => {
//                 // info!("Sending TxSyn, public_key: {}", public_key);
//             }
//             Err(err) => {
//                 info!("Failed to handle TxHashAck, err: {}", err,);
//             }
//         }
//     }
// }

pub(super) async fn handle_tx_syn(
    tx_syn: TxSynMsg,
    machine: &Machine,
) -> Result<(), SaksahaError> {
    machine
        .blockchain
        .dist_ledger
        .apis
        .insert_into_pool(tx_syn.tx_candidates)
        .await;

    Ok(())
}
