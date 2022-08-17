use crate::{machine::Machine, node::SaksahaNodeError, SaksahaError};
use futures::{stream::SplitSink, SinkExt};
use log::{debug, info, warn};
use sak_p2p_transport::{
    BlockAckMsg, BlockHashSyncMsg, BlockSynMsg, Msg, RecvReceipt, SendReceipt,
    TxHashSyncMsg, TxSynMsg, UpgradedConn, UpgradedP2PCodec,
};
use sak_types::{BlockHash, BlockHeight};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLockWriteGuard};

pub(in crate::node) async fn send_block_syn(
    mut conn_lock: RwLockWriteGuard<'_, UpgradedConn>,
    new_blocks: Vec<(BlockHeight, BlockHash)>,
    machine: &Arc<Machine>,
) -> Result<RecvReceipt, SaksahaNodeError> {
    let block_hashes: Vec<&BlockHash> = new_blocks
        .iter()
        .map(|(_, block_hash)| block_hash)
        .collect();

    let blocks = machine
        .blockchain
        .dist_ledger
        .apis
        .get_blocks(block_hashes)
        .await?;

    let mut blocks_to_send = Vec::with_capacity(blocks.len());

    for block in blocks {
        let txs = machine
            .blockchain
            .dist_ledger
            .apis
            .get_txs(&block.tx_hashes)
            .await?;

        blocks_to_send.push((block, txs));
    }

    conn_lock
        .send(Msg::BlockSyn(BlockSynMsg {
            blocks: blocks_to_send,
        }))
        .await?;

    let (msg, receipt) = conn_lock.next_msg().await;

    let msg = msg.ok_or(format!("block syn needs to be followed by ack"))??;

    println!("recv block ack");

    let block_ack_msg = match msg {
        Msg::BlockAck(m) => m,
        _ => {
            return Err(
                format!("Only block ack should arrive at this point").into()
            );
        }
    };

    Ok(receipt)
}

pub(in crate::node) async fn recv_block_syn(
    block_syn_msg: BlockSynMsg,
    machine: &Arc<Machine>,
    mut conn: RwLockWriteGuard<'_, UpgradedConn>,
) -> Result<SendReceipt, SaksahaNodeError> {
    println!("recv block syn");

    let blocks = block_syn_msg.blocks;

    let latest_block_height = machine
        .blockchain
        .dist_ledger
        .apis
        .get_latest_block_height()?
        .unwrap_or(0);

    for (block, txs) in blocks {
        if block.block_height != (latest_block_height + 1) {
            return Err("received not continuous block height".into());
        }

        machine
            .blockchain
            .dist_ledger
            .apis
            .sync_block(block, txs)
            .await?;
    }

    let block_ack_msg = Msg::BlockAck(BlockAckMsg {});

    let receipt = conn.send(block_ack_msg).await?;

    Ok(receipt)
}

pub(super) async fn handle_block_hash_syn<'a>(
    block_hash_syn_msg: BlockHashSyncMsg,
    machine: &Machine,
    conn: &'a mut RwLockWriteGuard<'_, UpgradedConn>,
) -> Result<(), SaksahaError> {
    let new_blocks = block_hash_syn_msg.new_blocks;

    let (_, latest_block_hash) = machine
        .blockchain
        .dist_ledger
        .apis
        .get_latest_block_hash()
        .await?
        .ok_or("height does not exist")?;

    debug!(
        "handle block hash syn, latest_block_hash: {}, received_new_blocks: {:?}",
        latest_block_hash,
        new_blocks,
    );

    let mut blocks_to_req = vec![];
    for (height, block_hash) in new_blocks {
        if block_hash != latest_block_hash {
            blocks_to_req.push((height, block_hash));
        }
    }

    match conn
        .send(Msg::BlockHashAck(BlockHashSyncMsg {
            new_blocks: blocks_to_req,
        }))
        .await
    {
        Ok(_) => {}
        Err(err) => {
            warn!("Failed to handle BlockHashSyn, err: {}", err,);
        }
    };

    Ok(())
}

pub(super) async fn handle_block_syn(
    block_syn_msg: BlockSynMsg,
    machine: &Machine,
) -> Result<(), SaksahaError> {
    let blocks = block_syn_msg.blocks;

    let latest_block_height = machine
        .blockchain
        .dist_ledger
        .apis
        .get_latest_block_height()?
        .unwrap_or(0);

    for (block, txs) in blocks {
        if block.block_height != (latest_block_height + 1) {
            return Err("received not continuous block height".into());
        }

        machine
            .blockchain
            .dist_ledger
            .apis
            .sync_block(block, txs)
            .await?;
    }

    Ok(())
}

pub(super) async fn handle_block_hash_ack<'a>(
    block_hash_syn_msg: BlockHashSyncMsg,
    machine: &Machine,
    conn: &'a mut RwLockWriteGuard<'_, UpgradedConn>,
) -> Result<(), SaksahaError> {
    let new_blocks = block_hash_syn_msg.new_blocks;

    let block_hashes: Vec<&String> = new_blocks
        .iter()
        .map(|(_, block_hash)| block_hash)
        .collect();

    let blocks = machine
        .blockchain
        .dist_ledger
        .apis
        .get_blocks(block_hashes)
        .await?;

    let mut blocks_to_send = Vec::with_capacity(blocks.len());

    for block in blocks {
        let txs = machine
            .blockchain
            .dist_ledger
            .apis
            .get_txs(&block.tx_hashes)
            .await?;

        blocks_to_send.push((block, txs));
    }

    if !blocks_to_send.is_empty() {
        match conn
            .send(Msg::BlockSyn(BlockSynMsg {
                blocks: blocks_to_send,
            }))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("Failed to handle blockHashAck, err: {}", err,);
            }
        }
    }

    Ok(())
}
