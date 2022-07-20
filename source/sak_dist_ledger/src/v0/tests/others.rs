use super::utils;
use crate::SyncPool;
use sak_contract_std::Storage;
use sak_types::TxCandidate;

#[tokio::test(flavor = "multi_thread")]
#[should_panic]
async fn test_insert_invalid_contract_to_tx_pool() {
    sak_test_utils::init_test_log();
    sak_test_utils::init_test_config(&vec![String::from("test")]).unwrap();

    let test_wasm = include_bytes!("./test_invalid_contract.wasm").to_vec();

    let dummy_tx = TxCandidate::new_dummy_pour_m1_to_p3_p4();

    let sync_pool = SyncPool::new();

    sync_pool.insert_tx(dummy_tx).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_deploy_ctr_and_invoke_execute_and_query_when_dist_ledger_writes_new_blocks(
) {
    sak_test_utils::init_test_log();
    sak_test_utils::init_test_config(&vec![String::from("test")]).unwrap();

    let ctr_addr: &String = &"test_wasm".to_string();
    let test_wasm = include_bytes!("./test_valid_contract.wasm").to_vec();

    let dist_ledger = utils::make_dist_ledger().await;

    dist_ledger.run().await;

    println!("\n[+] Block1: Deploying test validator contract");

    dist_ledger
        .apis
        .write_block(utils::make_dummy_block_candidate_1())
        .await
        .expect("Block_1 must be written");

    println!("\n[+] Block2: Execute::add_validator");

    dist_ledger
        .apis
        .write_block(utils::make_dummy_block_candidate_with_execute_tx())
        .await
        .expect("Block_2 must be written");

    println!("\n[+] Block3: Query::get_validator");

    dist_ledger
        .apis
        .write_block(utils::make_dummy_block_candidate_with_query_tx())
        .await
        .expect("Block_3 must be written");

    {
        let result: Storage = dist_ledger
            .apis
            .get_ctr_state(ctr_addr)
            .await
            .unwrap()
            .unwrap();

        println!("[*] result: {:#?}", result);
    }
}