use crate::{common::SakResult, crypto};
use logger::log;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct PConfig {
    pub p2p: PersistedP2PConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistedP2PConfig {
    pub secret: String,
    pub public_key: String,
}

impl PConfig {
    pub fn of(config_path: Option<String>) -> SakResult<PConfig> {
        let config_path = match config_path {
            Some(c) => c,
            None => {
                log!(
                    DEBUG,
                    "Config path is not given, creating a new config\n"
                );

                let default_path = PConfig::get_default_path()?;
                if default_path.exists() {
                    log!(DEBUG, "Found a config at the default location\n");

                    return PConfig::load(default_path);
                } else {
                    return PConfig::new();
                }
            }
        };

        let config_path = PathBuf::from(config_path);
        PConfig::load(config_path)
    }

    fn new() -> SakResult<PConfig> {
        log!(DEBUG, "Creating a new config\n");

        let sk = crypto::generate_key();
        let (sk, pk) = crypto::encode_key_pair(sk);
        let pconf = PConfig {
            p2p: PersistedP2PConfig {
                secret: sk,
                public_key: pk,
            },
        };

        match pconf.persist() {
            Ok(_) => Ok(pconf),
            Err(err) => Err(err),
        }
    }
}
