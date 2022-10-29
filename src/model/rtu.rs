use std::fs;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use brewdrivers::drivers::InstrumentError;

use super::{validators, Device, RTUError};

/// A digital representation of an RTU
///
/// This is meant to be serialized from a configuration file. This is
/// also the data structure that is sent between the iris server and the front-end
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RTU {
    /// The RTU name, for display purposes
    pub name: String,
    /// The RTU id, must be unique among all RTUs and not contain whitespace
    pub id: String,
    /// The IP address of the RTU. Later, this may be generated for you, but
    /// for now it's manually set.
    pub ip_addr: Ipv4Addr,
    /// A list of devices connected to the RTU
    pub devices: Vec<Device>,
}

impl RTU {
    /// This calls [`Device::enact`](crate::model::Device::enact) on each device in the RTU
    pub async fn enact(&mut self) -> Result<(), InstrumentError> {
        for dev in self.devices.iter_mut() {
            dev.enact().await?;
        }
        Ok(())
    }

    /// This calls [`Device::update`](crate::model::Device::update) on each device in the RTU
    pub async fn update(&mut self) -> Result<(), InstrumentError> {
        for dev in self.devices.iter_mut() {
            dev.update().await?;
        }
        Ok(())
    }

    /// Reads the configuration file and builds an RTU from that. Note that while this method
    /// does take an optional file path, that's just used for testing purposes. You should pass
    /// `None` to this method and use the defualt configuration file at [`crate::CONFIG_FILE`](crate::CONFIG_FILE).
    ///
    /// This will fail if the RTU cannot be deserialized from the configuration file.
    ///
    /// This method calls [`RTU::validate()`](crate::model::RTU::validate) and returns an error if any of
    /// them don't succeed.
    pub fn generate(conf_path: Option<&str>) -> Result<RTU, RTUError> {
        let file_path = conf_path.or(Some(crate::CONFIG_FILE));

        // TODO: Get IPv4 here programatically instead of writing it in the file

        // Get the contents of the config file
        let file_contents = fs::read_to_string(
            // this is safe
            file_path.unwrap(),
        )
        .map_err(|err| RTUError::IOError(err))?;

        // Serialize the file. Return an Err if it doesn't succeed
        let rtu = serde_yaml::from_str::<RTU>(&file_contents)
            .map_err(|err| RTUError::SerdeParseError(err))?;

        rtu.validate()?;
        Ok(rtu)
    }

    /// Run all the [`validators`](crate::model::validators). Return an error if any of them don't succeed.
    pub fn validate(&self) -> Result<(), RTUError> {
        validators::devices_have_unique_ids(&self)?;
        validators::id_has_no_whitespace(&self)?;
        validators::serial_port_is_valid(&self)?;
        Ok(())
    }

}
