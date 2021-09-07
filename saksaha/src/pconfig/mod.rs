use crate::errors::Error;
use directories::ProjectDirs;
use logger::log;
use std::path::{Path, PathBuf};
use std::{fs};

pub mod parse;

static DEFAULT_CONFIG_FILE_NAME: &str = "config.json";

pub struct PConfig {
    pub p2p: PersistedP2PConfig,
}

pub struct PersistedP2PConfig {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
}

impl PConfig {
    pub fn new(path: Option<&str>) -> Result<Self, Error> {
        return load_or_create_config(path);
    }
}

fn load_or_create_config(path: Option<&str>) -> Result<PConfig, Error> {
    if let Some(p) = path {
        log!(DEBUG, "Config path is given, probing a path: {}\n", p);

        let path = PathBuf::from(p);

        if !path.exists() {
            return Error::result(format!("Config file does not exist"));
        }

        if path.is_dir() {
            return Error::result(format!(
                "Config path must be a file, not directory"
            ));
        }

        log!(DEBUG, "Found config file, loading...\n");
        return parse::from(path);
    } else {
        log!(
            DEBUG,
            "Config path is not given, creating the default one\n"
        );

        let app_path = create_or_get_app_path();

        if let Err(e) = app_path {
            return Error::result(format!(
                "Error setting up an app path, err: {}",
                e
            ));
        }

        let app_path = app_path.unwrap();
        let config_path = app_path.join(DEFAULT_CONFIG_FILE_NAME);

        if config_path.exists() {
            return parse::from(app_path);
        } else {
            return create_default_config(config_path);
        }
    }
}

fn create_or_get_app_path() -> Result<PathBuf, Error> {
    if let Some(dir) = ProjectDirs::from("com", "Saksaha", "Saksaha") {
        let app_path = dir.config_dir();
        if !app_path.exists() {
            match fs::create_dir(app_path) {
                Ok(_) => {
                    return Ok(app_path.to_path_buf());
                }
                Err(err) => {
                    return Error::result(format!(
                        "Error creating a path, {}",
                        err
                    ));
                }
            }
        }
        return Ok(app_path.to_path_buf());
    } else {
        return Error::result(format!("Error forming an app path"));
    }
}

fn create_default_config(config_path: PathBuf) -> Result<PConfig, Error> {
    return Error::result(format!("power"));
    // return Some(path);
}

#[cfg(test)]
mod test {
    use crate::testenv;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn it_creates_config_path() {
        testenv::run_test(|test_env| {
            let testdump = test_env
                .testdump
                .as_ref()
                .expect("Test dump path should be provided");

            let path = testdump.join("saksaha-config");
            let path = path.to_str().expect("Error making test config path");

            let _ = super::load_or_create_config(Some(path))
                .expect("Error creating config");

            // PathBuf::from(path_name);

            println!("{:?}", testdump);
        })
    }
}
