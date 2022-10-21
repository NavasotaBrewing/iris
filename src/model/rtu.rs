use std::fs;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
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
    #[error("Duplicate ID found: {0}")]
    DuplicateID(String),
    #[error("Invalid value for {{ `{key}`: `{value}` }}, {msg}")]
    InvalidValue {
        key: String,
        value: String,
        msg: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RTU {
    pub name: String,
    pub id: String,
    pub ip_addr: Ipv4Addr,
    pub devices: Vec<Device>,
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

    /// Reads the configuration file and builds an RTU from that
    pub fn generate(conf_path: Option<&str>) -> Result<RTU, RTUError> {
        let file_path = conf_path.or(Some(crate::CONFIG_FILE));

        // TODO: Get IPv4 here programatically instead of writing it in the file

        // Get the contents of the config file
        let file_contents = fs::read_to_string(
            // this is safe
            file_path.unwrap(),
        )
        .map_err(|err| RTUError::IOError(err))?;

        // Serialize the file. Return an Err is it doesn't succeed
        let rtu = serde_yaml::from_str::<RTU>(&file_contents)
            .map_err(|err| RTUError::SerdeParseError(err))?;

        // Run all the validators. Return an error if any of them doesn't succeed.
        validators::unique_ids(&rtu)?;
        validators::id_has_no_whitespace(&rtu)?;
        Ok(rtu)
    }
}

/// Validators for when the config is deserialized from the config file
mod validators {
    use log::error;
    use std::collections::HashMap;

    use super::{RTUError, RTU};

    pub fn unique_ids(rtu: &RTU) -> Result<(), RTUError> {
        let mut seen: HashMap<&String, bool> = HashMap::new();
        for device in &rtu.devices {
            if seen.get(&device.id).is_some() {
                error!("Found duplicate device ID `{}` in config file", device.id);
                error!("Rename the duplicate ID `{}` to something else", device.id);
                return Err(RTUError::DuplicateID(device.id.clone()));
            }
            seen.insert(&device.id, true);
        }
        Ok(())
    }

    pub fn id_has_no_whitespace(rtu: &RTU) -> Result<(), RTUError> {
        if rtu.id.contains(char::is_whitespace) {
            return Err(RTUError::InvalidValue {
                key: format!("id"),
                value: format!("{}", rtu.id),
                msg: format!("RTU ID cannot contain whitespace"),
            });
        }

        for dev in &rtu.devices {
            if dev.id.contains(char::is_whitespace) {
                return Err(RTUError::InvalidValue {
                    key: format!("id"),
                    value: format!("{}", &dev.id),
                    msg: format!("Device ID cannot contain whitespace"),
                });
            }
        }

        Ok(())
    }
}
