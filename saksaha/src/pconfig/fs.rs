use crate::common::{
    SakResult,
    ErrorKind,
};
use crate::pconfig::PConfig;
use crate::{err_res, err_resk};
use directories::ProjectDirs;
use logger::log;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "config.json";

pub struct FS;

impl FS {
    pub fn new() -> FS {
        FS {}
    }

    pub fn persist(pconfig: PConfig) -> SakResult<PConfig> {
        let serialized = match serde_json::to_string_pretty(&pconfig) {
            Ok(s) => s,
            Err(err) => {
                return err_res!("Cannot serialize configuration, err: {}", err);
            }
        };

        let app_path = create_or_get_app_path()?;
        let config_path = app_path.join(CONFIG_FILE_NAME).to_owned();

        if config_path.exists() {
            return err_res!("Config file already exists, something is wrong");
        }

        log!(DEBUG, "Writing a config, at: {:?}\n", config_path);

        match fs::write(config_path.to_owned(), serialized) {
            Ok(_) => {
                Ok(pconfig)
            },
            Err(err) => err_res!("Error writing the config, err: {}", err),
        }
    }

    pub fn load(path: PathBuf) -> SakResult<PConfig> {
        log!(DEBUG, "Load configuration, path: {:?}\n", path);

        if !path.exists() {
            return err_resk!(
                ErrorKind::FileNotExist,
                "Config does not exist at path: {:?}\n",
                path
            );
        }

        let file = match fs::read_to_string(path.to_owned()) {
            Ok(f) => f,
            Err(err) => {
                return err_res!(
                    "Error reading file, path: {:?}, err: {}",
                    path,
                    err
                );
            }
        };

        match serde_json::from_str(file.as_str()) {
            Ok(pconf) => return Ok(pconf),
            Err(err) => {
                return err_res!("Error deserializing config, err: {}", err);
            }
        }
    }

    pub fn get_default_path() -> SakResult<PathBuf> {
        let app_path = create_or_get_app_path()?;
        let config_path = app_path.join(CONFIG_FILE_NAME);

        Ok(config_path)
    }
}

fn create_or_get_app_path() -> SakResult<PathBuf> {
    if let Some(dir) = ProjectDirs::from("com", "Saksaha", "Saksaha") {
        let app_path = dir.config_dir();
        if !app_path.exists() {
            match fs::create_dir(app_path) {
                Ok(_) => {
                    return Ok(app_path.to_path_buf());
                }
                Err(err) => {
                    return err_res!("Error creating a path, {}", err);
                }
            }
        }
        return Ok(app_path.to_path_buf());
    } else {
        return err_res!("Error forming an app path");
    }
}
