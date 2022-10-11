use std::{fs, path::{Path, PathBuf}};

use serde::{Serialize, Deserialize};
use thiserror::Error;

use super::{Device, Mode};

#[derive(Error, Debug)]
pub enum RTUError {
    #[error("Configuration file not found at /etc/NavasotaBrewing/rtu_conf.yaml")]
    FileNotFound,
    #[error("IO error: {0}")]
    IOError(std::io::Error),
    #[error("Permission error, cannot access /etc/NavasotaBrewing/rtu_conf.yaml")]
    PermissionError,
    #[error("Serde parse error: {0}")]
    SerdeParseError(serde_yaml::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RTU {
    pub name: String,
    pub id: String,
    pub devices: Vec<Device>
}

impl RTU {
    pub async fn update(rtu: &mut RTU, mode: &Mode) {
        for mut device in &mut rtu.devices {
            Device::update(&mut device, &mode).await;
        }
    }

    /// Reads the configuration file at /etc/NavasotaBrewing/rtu_conf.yaml and builds an RTU from that
    pub fn generate(conf_path: Option<&str>) -> Result<RTU, RTUError> {
        let file_contents = fs::read_to_string(
            conf_path.or(Some("/etc/NavasotaBrewing/rtu_conf.yaml")).unwrap()
        ).map_err(|err| RTUError::IOError(err) )?;
        serde_yaml::from_str::<RTU>(&file_contents).map_err(|err| RTUError::SerdeParseError(err) )
    }
}