use crate::{mock_pos, DistLedger, DistLedgerArgs};
use sak_types::BlockCandidate;

const APP_NAME: &str = "saksaha";

pub async fn mock_dist_ledger(block: BlockCandidate) -> DistLedger {
    let pos = mock_pos();

    let ledger_path = {
        let config_dir = sak_fs::get_config_dir(APP_NAME).unwrap();
        config_dir.join("db/ledger")
    };

    let dist_ledger_args = DistLedgerArgs {
        // public_key: String::from("test"),
        tx_sync_interval: None,
        genesis_block: Some(block),
        consensus: pos,
        block_sync_interval: None,
        ledger_path,
    };

    let dist_ledger = DistLedger::init(dist_ledger_args)
        .await
        .expect("Blockchain should be initialized");

    dist_ledger
}

pub async fn mock_dist_ledger_1() -> DistLedger {
    let pos = mock_pos();

    let ledger_path = {
        let config_dir = sak_fs::get_config_dir(APP_NAME).unwrap();
        config_dir.join("db/ledger")
    };

    let dist_ledger_args = DistLedgerArgs {
        // public_key: String::from("test"),
        tx_sync_interval: None,
        genesis_block: Some(sak_types::mock_block_1()),
        consensus: pos,
        block_sync_interval: None,
        ledger_path,
    };

    let dist_ledger = DistLedger::init(dist_ledger_args)
        .await
        .expect("Blockchain should be initialized");

    dist_ledger
}
