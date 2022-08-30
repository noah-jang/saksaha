use super::{ProfiledConfig, ProfiledP2PConfig};
use crate::config::{NodeConfig, RPCConfig};
use sak_p2p_addr::{AddrStatus, UnknownAddr};

pub(super) fn dev_local_1() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_1"),
        p2p: ProfiledP2PConfig {
            disc_port: Some(35518),
            secret: Some(String::from(
                "7297b903877a957748b74068d63d6d5661481975240\
                99fc1df5cd9e8814c66c7",
            )),
            public_key_str: Some(String::from(
                "045739d074b8722891c307e8e75c9607e0b55a80778\
                b42ef5f4640d4949dbf3992f6083b729baef9e9545c4\
                e95590616fd382662a09653f2a966ff524989ae8c0f",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35518,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: true,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig {
            rpc_port: Some(34418),
        },
    };
}

pub(super) fn dev_local_2() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_2"),
        p2p: ProfiledP2PConfig {
            disc_port: Some(35519),
            secret: Some(String::from(
                "224d0898389759f29ad5c9a6472b26fff86b6293889\
                88eec457a88ce50e907a0",
            )),
            public_key_str: Some(String::from(
                "042c8d005bd935597117181d8ceceaef6d1162de78c32856\
                89d0c36c6170634c124f7b9b911553a1f483ec565c199ea29ff1\
                cd641f10c9a5f8c7c4d4a026db6f7b",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35518,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig {
            rpc_port: Some(34419),
        },
    };
}

pub(super) fn dev_local_3() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_3"),
        p2p: ProfiledP2PConfig {
            disc_port: None,
            secret: Some(String::from(
                "ca4a030e5eeb0ced180faf7705ed1069587f9236e620f582\
                08ec79e9e93deedf",
            )),
            public_key_str: Some(String::from(
                "04014eca9f9ed3ba1518e49ac6fc0ffab1825f539f09f632\
                661d989ddf535053fdaa8f3bbdefe626db05e338a6218db92\
                0f7d5b34461cd71cb9327da011e97d60f",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35518,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig { rpc_port: None },
    };
}

pub(super) fn dev_local_4() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_4"),
        p2p: ProfiledP2PConfig {
            disc_port: None,
            secret: Some(String::from(
                "07d1cf4bef6c931feba459730fc76c25ba444c1a1a498bd6\
                46c81fff797d187a",
            )),
            public_key_str: Some(String::from(
                "04452fcaa76dbbcd3a9e643fda1de97610e2468e14952d2f\
                770805e10d53d995c9d184cb3d0d8ca97924ff94c323166e0\
                c1c1d8eb16bfead486238ef299dc2578b",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35518,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig { rpc_port: None },
    };
}

pub(super) fn dev_local_5() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_5"),
        p2p: ProfiledP2PConfig {
            disc_port: None,
            secret: Some(String::from(
                "740dfe88ddd80fdc7509f251e66e6ed66be63d0ba27be867\
                23d467cb9473e8a4",
            )),
            public_key_str: Some(String::from(
                "045781a9434e205bd7993fabc79e00411cd42bff8ecb863c\
                8df60f08ffa5c1a28474f6837dc573a3b2bce8a087ac77ab5\
                24490858de7de8b32321f3850112a5e26",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35519,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig { rpc_port: None },
    };
}

pub(super) fn dev_local_6() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_6"),
        p2p: ProfiledP2PConfig {
            disc_port: Some(35520),
            secret: Some(String::from(
                "af23c891038bd9444377fb87803a858f1b01a53ff0989da3\
                6ff07e757256f32b",
            )),
            public_key_str: Some(String::from(
                "04616a281bc070acdc2c33bea02cfb99398a203dc7bae9da\
                0d1406bd2a9ec79bf2268bb4cd02faa4f3e9cf0235ef492d7\
                e0d2d43333a1716e986f63d9928b6c037",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35519,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig { rpc_port: None },
    };
}

pub(super) fn dev_local_7() -> ProfiledConfig {
    return ProfiledConfig {
        app_prefix: String::from("dev_local_7"),
        p2p: ProfiledP2PConfig {
            disc_port: None,
            secret: Some(String::from(
                "9a1d37e31d8f10d21d05c441d0c863fb7c17514e3993f954\
                655830bd7f7cefd8",
            )),
            public_key_str: Some(String::from(
                "042d2346593a3a502414b56d043826ea52190417537e0627a\
                ae1f02f91667bbc131b36acb331218e8584ea617343d6293e1\
                fb2bfe1968629975316536492ea6c80",
            )),
            bootstrap_addrs: vec![UnknownAddr {
                ip: String::from("127.0.0.1"),
                disc_port: 35520,
                p2p_port: None,
                sig: None,
                public_key_str: Some(String::from(
                    "\
                            04715796a40b0d58fc14a3c4ebee21cb\
                            806763066a7f1a17adbc256999764443\
                            beb8109cfd000718535c5aa27513a2ed\
                            afc6e8bdbe7c27edc2980f9bbc25142fc5\
                            ",
                )),
                status: AddrStatus::Initialized,
            }],
        },
        node: NodeConfig {
            miner: false,
            mine_interval: None,
            node_task_min_interval: None,
            peer_register_interval: None,
        },
        rpc: RPCConfig { rpc_port: None },
    };
}
