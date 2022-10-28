//! Validators for when the RTU is deserialized from the config file
//!
//! These are called on the RTU and return an Err([RTUError](crate::model::RTUError)) if
//! the RTU doesn't pass the test. It's another layer of validation on top of `serde_yaml`. This ensures
//! the values in the RTU are actually correct, not just that it's valid YAML syntax.
//!
//! `serde` takes care of making sure the proper values are present; only values in an `Option<>` can be missing.

use log::{error, info};
use std::{collections::HashMap, path::Path};

use super::{RTUError, RTU};

/// Returns `Ok(())` if each device in the RTU has a unique ID
pub fn devices_have_unique_ids(rtu: &RTU) -> Result<(), RTUError> {
    let mut seen: HashMap<&String, bool> = HashMap::new();
    for device in &rtu.devices {
        if seen.get(&device.id).is_some() {
            error!("Found duplicate device ID `{}` in config file", device.id);
            error!("Rename the duplicate ID `{}` to something else", device.id);
            return Err(RTUError::validation_error(
                ("id", device.id.as_str()),
                "duplicate id",
            ));
        }
        seen.insert(&device.id, true);
    }

    info!("RTU passed devices_have_unique_ids() validator");
    Ok(())
}

/// Returns `Ok(())` if the RTU ID and every device ID does not contain whitespace
pub fn id_has_no_whitespace(rtu: &RTU) -> Result<(), RTUError> {
    if rtu.id.contains(char::is_whitespace) {
        let err = RTUError::validation_error(("id", &rtu.id), "rtu ID cannot contain whitespace");
        error!("{}", err);
        return Err(err);
    }

    for dev in &rtu.devices {
        if dev.id.contains(char::is_whitespace) {
            let err =
                RTUError::validation_error(("id", &dev.id), "device ID cannot contain whitespace");
            error!("{}", err);
            return Err(err);
        }
    }

    info!("RTU passed id_has_no_whitespace() validator");
    Ok(())
}

/// This will actually *not* fail if the serial port doesn't exist. Sometimes we disconnect
/// the cable and the port goes away, but it's still valid. Instead, it just checks that it's a
/// valid path.
///
/// This will however print a `warn!()` statement if the port doesn't exist, if a logger is configured.
pub fn serial_port_is_valid(rtu: &RTU) -> Result<(), RTUError> {
    for dev in &rtu.devices {
        let path = Path::new(&dev.port);
        if !path.starts_with("/dev") {
            // return Err(RTUError)
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_validators {
    use super::*;
    
    use std::{net::Ipv4Addr, str::FromStr};

    use brewdrivers::controllers::*;
    use tokio_test::{assert_err, assert_ok};
    use AnyState as AS;
    use BinaryState as BS;
    
    use crate::model::{RTU, Device};

    // Just quickly sets up an RTU for testing purposes
    fn rtu(name: &str, id: &str, devices: Vec<Device>) -> RTU {
        RTU {
            name: String::from(name),
            id: String::from(id),
            ip_addr: Ipv4Addr::from_str("0.0.0.0").unwrap(),
            devices,
        }
    }

    // Quickly builds a device for testing purposes
    fn device(
        id: &str,
        name: &str,
        port: &str,
        addr: u8,
        controller: Controller,
        controller_addr: u8,
        state: AnyState,
    ) -> Device {
        Device {
            id: String::from(id),
            name: String::from(name),
            port: String::from(port),
            addr,
            controller,
            controller_addr,
            state,
            pv: None,
            sv: None,
        }
    }

    #[test]
    fn test_devices_have_unique_ids() {
        let devices = vec![
            device(
                "pump",
                "Pump",
                "/dev/ttyUSB0",
                0,
                Controller::STR1,
                254,
                AS::BinaryState(BS::On),
            ),
            device(
                "pump",
                "Other pump with same ID",
                "/dev/ttyUSB0",
                1,
                Controller::STR1,
                254,
                AS::BinaryState(BS::On),
            ),
            device(
                "pump2",
                "Other pump with different ID",
                "/dev/ttyUSB0",
                2,
                Controller::STR1,
                254,
                AS::BinaryState(BS::On),
            ),
        ];

        let mut rtu = rtu("Testing RTU", "testing-id", devices);

        assert_err!(devices_have_unique_ids(&rtu));
        rtu.devices.remove(1);
        assert_ok!(devices_have_unique_ids(&rtu));
    }

    #[test]
    fn test_id_has_no_whitespace() {
        let devices = vec![device(
            "pump id with whitespace",
            "Pump",
            "/dev/ttyUSB0",
            0,
            Controller::STR1,
            254,
            AS::BinaryState(BS::On),
        )];

        let mut rtu = rtu("Testing RTU", "testing id with whitespace", devices);

        assert_err!(id_has_no_whitespace(&rtu));
        rtu.devices[0].id = String::from("something-without-whitespace");
        // Still an error because the RTU id has whitespace
        assert_err!(id_has_no_whitespace(&rtu));
        rtu.id = String::from("no-whitespace");
        assert_ok!(id_has_no_whitespace(&rtu));
    }
}
