use super::{
    ARG_CH_ID, ARG_DST_PK, ARG_SERIALIZED_INPUT, DUMMY_CHANNEL_ID_1,
    DUMMY_CHANNEL_ID_2, DUMMY_CHANNEL_ID_3, ENVELOPE_CONTRACT,
    INIT_CHANNEL_ID_1, STORAGE_CAP,
};

use envelope_contract::{
    request_type::OPEN_CH, Channel, EnvelopeStorage, GetChListParams,
    GetMsgParams, OpenChParams, SendMsgParams,
};
use sak_contract_std::{CtrCallType, Request, Storage};
use sak_vm::{CtrFn, VM};
use std::collections::HashMap;

// pub(crate) struct OpenChInput {
//     eph_pk: String,
//     ch_id: String,
//     sign: String,
//     chat: String,
// }

fn get_single_message() -> String {
    String::from("Hello! I belong to saksaha")
}

fn get_multi_messages() -> Vec<String> {
    vec![
        String::from("Hi, there"),
        String::from("This is a secret message"),
    ]
}

fn get_her_pk() -> String {
    String::from("her_pk12345")
}

fn make_mock_storage(msgs: &Vec<String>) -> Storage {
    let mut open_ch_reqs = HashMap::new();
    // open_ch_reqs.insert(
    //     get_her_pk(),
    //     vec![
    //         Channel {
    //             ch_id: "ch_id_1".to_string(),
    //             eph_key: "eph_key_1".to_string(),
    //             sig: "sig_1".to_string(),
    //         },
    //         Channel {
    //             ch_id: DUMMY_CHANNEL_ID_1.to_string(),
    //             eph_key: "eph_key_1".to_string(),
    //             sig: "sig_1".to_string(),
    //         },
    //     ],
    // );

    let mut chats = HashMap::new();
    chats.insert(DUMMY_CHANNEL_ID_1.to_string(), msgs.clone());

    let envelope_storage = EnvelopeStorage {
        open_ch_reqs,
        chats,
    };

    // let mut ret = Storage::with_capacity(STORAGE_CAP);

    // let key = String::from(DUMMY_CHANNEL_ID_1);

    // let value = serde_json::to_string(&msgs).unwrap();

    // ret.insert(key, value);

    // let key = String::from(get_her_pk());

    // let input: Vec<String> = vec![
    //     String::default(),
    //     DUMMY_CHANNEL_ID_1.to_string(),
    //     String::default(),
    //     String::default(),
    // ];

    // let input = serde_json::to_string(&input).unwrap();

    // let input_vec: Vec<String> = vec![input];

    // let value = serde_json::to_string(&input_vec).unwrap();

    // ret.insert(key, value);

    serde_json::to_vec(&envelope_storage).unwrap()
}

fn make_mock_open_ch() -> Channel {
    Channel {
        ch_id: DUMMY_CHANNEL_ID_2.to_string(),
        eph_key: String::default(),
        sig: String::default(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_messenger_get_msgs() {
    sak_test_utils::init_test_log();
    let vm = VM::init().expect("VM should be initiated");

    let test_dummy_messege = get_multi_messages();

    let messages_state = make_mock_storage(&test_dummy_messege);

    let request = {
        let get_msg_params = GetMsgParams {
            ch_id: DUMMY_CHANNEL_ID_1.to_string(),
        };

        let args = serde_json::to_vec(&get_msg_params).unwrap();

        Request {
            req_type: "get_msgs".to_string(),
            args,
            ctr_call_type: CtrCallType::Query,
        }
    };

    {
        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Query(request, messages_state);

        let receipt = vm
            .invoke(ctr_wasm, ctr_fn)
            .expect("message should be obtained");

        let result = receipt.result;

        let chats: Vec<String> = serde_json::from_slice(&result).unwrap();

        println!("messages expected: {:?}", test_dummy_messege);

        println!("messages acquired: {:?}", chats);

        assert_eq!(test_dummy_messege, chats);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_messenger_get_ch_list() {
    sak_test_utils::init_test_log();
    let vm = VM::init().expect("VM should be initiated");

    let her_pk = get_her_pk();

    let dummy_messeges = get_multi_messages();

    let (request, storage) = {
        // let mut args = HashMap::with_capacity(2);
        // args.insert(String::from(ARG_DST_PK), her_pk.clone());

        let get_ch_list_params = GetChListParams {
            dst_pk: her_pk.clone(),
        };

        let args = serde_json::to_vec(&get_ch_list_params).unwrap();

        let req = Request {
            req_type: String::from("get_ch_list"),
            args,
            ctr_call_type: CtrCallType::Query,
        };

        let storage = make_mock_storage(&dummy_messeges);

        (req, storage)
    };

    {
        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Query(request, storage);

        let receipt = vm.invoke(ctr_wasm, ctr_fn).unwrap();

        let open_ch_data_vec: Vec<Channel> =
            serde_json::from_slice(&receipt.result).unwrap();

        println!("expected channel id : {:?}", vec![DUMMY_CHANNEL_ID_1]);

        println!("updated channel id: {:?}", open_ch_data_vec);

        assert_eq!(
            vec![DUMMY_CHANNEL_ID_1],
            vec![open_ch_data_vec[1].ch_id.to_owned()]
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_messenger_open_channel() {
    sak_test_utils::init_test_log();
    let vm = VM::init().expect("VM should be initiated");

    let new_pk = "abcdef".to_string();

    let dummy_messeges = get_multi_messages();

    let Channel {
        ch_id,
        eph_key,
        sig,
    } = make_mock_open_ch();

    let (request, storage) = {
        let open_ch_params = OpenChParams {
            dst_pk: new_pk.clone(),
            open_ch: Channel {
                ch_id,
                eph_key,
                sig,
            },
        };

        let args = serde_json::to_vec(&open_ch_params).unwrap();

        let req = Request {
            req_type: OPEN_CH.to_string(),
            args,
            ctr_call_type: CtrCallType::Execute,
        };

        let storage = make_mock_storage(&dummy_messeges);

        (req, storage)
    };

    {
        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Execute(request, storage);

        let receipt = vm.invoke(ctr_wasm, ctr_fn).unwrap();

        let updated_storage = receipt
            .updated_storage
            .ok_or("State needs to be updated, ")
            .unwrap();

        let storage: EnvelopeStorage =
            serde_json::from_slice(&updated_storage).unwrap();

        // let envelope_storage: EnvelopeStorage =
        //     serde_json::from_slice(&storage).unwrap();

        // let open_ch_reqs = envelope_storage.open_ch_reqs.get(&new_pk).unwrap();
        // let open_ch_reqs = storage.open_ch_reqs.get(&new_pk).unwrap();

        // let open_ch = open_ch_reqs.get(0).unwrap();

        println!("expected channel id : {:?}", DUMMY_CHANNEL_ID_2);
        println!("updated channel id: {:?}", storage);

        // let ch_id = &open_ch.ch_id;

        // assert_eq!(DUMMY_CHANNEL_ID_2, ch_id);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_messenger_send_msg() {
    sak_test_utils::init_test_log();
    let vm = VM::init().expect("VM should be initiated");

    let dummy_messeges = get_multi_messages();

    let expected_msg = get_single_message();

    let (request, storage) = {
        let mut args = HashMap::with_capacity(2);
        args.insert(String::from(ARG_CH_ID), String::from(DUMMY_CHANNEL_ID_3));
        args.insert(String::from(ARG_SERIALIZED_INPUT), expected_msg.clone());

        let send_msg_params = SendMsgParams {
            ch_id: String::from(DUMMY_CHANNEL_ID_3),
            msg: expected_msg.clone(),
        };

        let args = serde_json::to_vec(&send_msg_params).unwrap();

        let req = Request {
            req_type: String::from("send_msg"),
            args,
            ctr_call_type: CtrCallType::Execute,
        };

        let storage = make_mock_storage(&dummy_messeges);

        (req, storage)
    };

    {
        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Execute(request, storage);

        let receipt = vm
            .invoke(ctr_wasm, ctr_fn)
            .expect("State should be obtained");

        let updated_storage = receipt.updated_storage.unwrap();

        println!(
            "updated_storage: {:#?}",
            String::from_utf8(updated_storage.clone())
        );

        let storage: Storage =
            serde_json::from_slice(&updated_storage).unwrap();

        let envelope_storage: EnvelopeStorage =
            serde_json::from_slice(&storage).unwrap();

        let chats = envelope_storage.chats.get(DUMMY_CHANNEL_ID_3).unwrap();

        let msg = chats.get(0).unwrap();

        println!("expected msg: {:?}", expected_msg);

        println!("updated msg: {:?}", msg);

        assert_eq!(&expected_msg, msg);
    };
}

#[tokio::test(flavor = "multi_thread")]
async fn test_messenger_open_channel_me_and_you() {
    sak_test_utils::init_test_log();
    let vm = VM::init().expect("VM should be initiated");

    let my_pk = "my_pk".to_string();
    let your_pk = "your_pk".to_string();

    let mut open_ch_reqs = HashMap::new();

    let mut chats = HashMap::new();
    chats.insert(DUMMY_CHANNEL_ID_1.to_string(), vec![String::from("")]);

    let envelope_storage = EnvelopeStorage {
        open_ch_reqs,
        chats,
    };

    let storage = serde_json::to_vec(&envelope_storage).unwrap();

    let Channel {
        ch_id,
        eph_key,
        sig,
    } = make_mock_open_ch();

    let receipt_1 = {
        let open_ch_params = OpenChParams {
            dst_pk: my_pk.clone(),
            open_ch: Channel {
                ch_id: ch_id.clone(),
                eph_key: eph_key.clone(),
                sig: sig.clone(),
            },
        };

        let args = serde_json::to_vec(&open_ch_params).unwrap();

        let request = Request {
            req_type: OPEN_CH.to_string(),
            args,
            ctr_call_type: CtrCallType::Execute,
        };

        // let storage = make_mock_storage(&dummy_messeges);

        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Execute(request, storage.clone());

        vm.invoke(ctr_wasm, ctr_fn).unwrap()
    };

    let receipt_2 = {
        let open_ch_params = OpenChParams {
            dst_pk: your_pk.clone(),
            open_ch: Channel {
                ch_id,
                eph_key,
                sig,
            },
        };

        let args = serde_json::to_vec(&open_ch_params).unwrap();

        let request = Request {
            req_type: OPEN_CH.to_string(),
            args,
            ctr_call_type: CtrCallType::Execute,
        };

        let ctr_wasm = ENVELOPE_CONTRACT.to_vec();
        let ctr_fn = CtrFn::Execute(request, storage);

        vm.invoke(ctr_wasm, ctr_fn).unwrap()
    };

    {
        let updated_storage_1 = receipt_1
            .updated_storage
            .ok_or("State needs to be updated, ")
            .unwrap();

        let storage_1: EnvelopeStorage =
            serde_json::from_slice(&updated_storage_1).unwrap();

        let updated_storage_2 = receipt_2
            .updated_storage
            .ok_or("State needs to be updated, ")
            .unwrap();

        let storage_2: EnvelopeStorage =
            serde_json::from_slice(&updated_storage_2).unwrap();

        println!("updated channel_1 id: {:?}", storage_1);
        println!("updated channel_2 id: {:?}", storage_2);
        let open_ch_reqs_1 = storage_1.open_ch_reqs.get(&my_pk).unwrap().len();
        let open_ch_reqs_2 =
            storage_2.open_ch_reqs.get(&your_pk).unwrap().len();

        assert_eq!(open_ch_reqs_1, 1);
        assert_eq!(open_ch_reqs_2, 1);
    }
}
