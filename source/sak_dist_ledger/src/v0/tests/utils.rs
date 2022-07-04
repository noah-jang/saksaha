use sak_contract_std::{CtrCallType, Request};
use sak_types::{BlockCandidate, Tx};
use std::collections::HashMap;

pub(crate) fn make_dummy_block_candidate_1() -> Option<BlockCandidate> {
    let test_wasm = include_bytes!("./test_valid_contract.wasm").to_vec();

    let block_candidate: BlockCandidate = {
        let dummy_ctr_deploying_tx = Tx::new(
            String::from("1346546123"),
            test_wasm,
            String::from("0x111"),
            b"0x1111".to_vec(),
            Some(String::from("test_wasm")),
        );

        BlockCandidate {
            validator_sig: String::from("Ox6a03c8sbfaf3cb06"),
            transactions: vec![dummy_ctr_deploying_tx],
            witness_sigs: vec![String::from("1"), String::from("2")],
            created_at: String::from("2022061515340000"),
            height: 0,
        }
    };

    Some(block_candidate)
}

pub(crate) fn make_dummy_block_candidate_with_query_tx(
) -> Option<BlockCandidate> {
    let block_candidate: BlockCandidate = {
        let dummy_ctr_calling_query_tx: Tx = {
            let request_query_get_validator: Request = {
                Request {
                    req_type: "get_validator".to_string(),
                    arg: HashMap::with_capacity(10),
                    ctr_call_type: CtrCallType::Query,
                }
            };

            Tx::new(
                String::from("0973948293"),
                serde_json::to_string(&request_query_get_validator)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                String::from("0x222"),
                b"0x1111".to_vec(),
                Some(String::from("test_wasm")),
            )
        };

        BlockCandidate {
            validator_sig: String::from("Ox6a03c8sbfaf3cb06"),
            transactions: vec![dummy_ctr_calling_query_tx],
            witness_sigs: vec![String::from("3"), String::from("4")],
            created_at: String::from("2022061515340000"),
            height: 1,
        }
    };

    Some(block_candidate)
}

pub(crate) fn make_dummy_block_candidate_with_execute_tx(
) -> Option<BlockCandidate> {
    let block_candidate = {
        let dummy_validator_1 = String::from(
            "\
                    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\
                    ccccccccccccccccccccccccccccccccc\
                    2222222222222222222222222222222\
                ",
        );

        let mut arg = HashMap::with_capacity(10);
        arg.insert(String::from("validator"), dummy_validator_1);

        let request_execute_add_validator_1 = Request {
            req_type: "add_validator".to_string(),
            arg,
            ctr_call_type: CtrCallType::Execute,
        };

        let dummy_ctr_calling_execute_add_validator_tx_1: Tx = {
            Tx::new(
                String::from("3479422851"),
                serde_json::to_string(&request_execute_add_validator_1)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                String::from("0x444"),
                b"0x1111".to_vec(),
                Some(String::from("test_wasm")),
            )
        };

        let dummy_validator_2 = String::from(
            "\
                    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\
                    ccccccccccccccccccccccccccccccccc\
                    3333333333333333333333333333333\
                ",
        );

        let mut arg = HashMap::with_capacity(10);
        arg.insert(String::from("validator"), dummy_validator_2);

        let request_execute_add_validator_2 = Request {
            req_type: "add_validator".to_string(),
            arg,
            ctr_call_type: CtrCallType::Execute,
        };

        let dummy_ctr_calling_execute_add_validator_tx_2: Tx = {
            Tx::new(
                String::from("3479422851"),
                serde_json::to_string(&request_execute_add_validator_2)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                String::from("0x444"),
                b"0x1111".to_vec(),
                Some(String::from("test_wasm")),
            )
        };

        BlockCandidate {
            validator_sig: String::from("Ox6a03c8sbfaf3cb06"),
            transactions: vec![
                //
                dummy_ctr_calling_execute_add_validator_tx_1,
                dummy_ctr_calling_execute_add_validator_tx_2,
            ],
            witness_sigs: vec![String::from("3"), String::from("4")],
            created_at: String::from("2022061515340000"),
            height: 2,
        }
    };

    Some(block_candidate)
}