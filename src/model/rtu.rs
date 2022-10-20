use std::fs;
use std::net::Ipv4Addr;

use serde::{Serialize, Deserialize};
use thiserror::Error;

use brewdrivers::drivers::InstrumentError;

use super::Device;

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
    pub ip_addr: Option<Ipv4Addr>,
    pub devices: Vec<Device>
}

impl RTU {
    pub async fn enact(rtu: &mut RTU) -> Result<(), InstrumentError> {
        for dev in rtu.devices.iter_mut() {
            dev.enact().await?;
        }
        Ok(())
    }

    pub async fn update(rtu: &mut RTU) -> Result<(), InstrumentError> {
        for dev in rtu.devices.iter_mut() {
            dev.update().await?;
        }
        Ok(())
    }

    /// Reads the configuration file at /etc/NavasotaBrewing/rtu_conf.yaml and builds an RTU from that
    pub fn generate(conf_path: Option<&str>) -> Result<RTU, RTUError> {
        let file_path = conf_path.or(Some(crate::CONFIG_FILE));
        // TODO: Get IPv4 programatically instead of writing it in the file
        let file_contents = fs::read_to_string(
            // this is safe
            file_path.unwrap()
        ).map_err(|err| RTUError::IOError(err) )?;
        serde_yaml::from_str::<RTU>(&file_contents).map_err(|err| RTUError::SerdeParseError(err) )
    }
}