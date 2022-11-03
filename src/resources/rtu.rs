
use std::future::Future;

use crate::RTUState;
use brewdrivers::drivers::InstrumentError;
use gotham::prelude::*;
use gotham::state::State;
use gotham_restful::*;
use hyper::{Method, StatusCode};
use log::info;

use crate::error::RequestError;
use crate::resp::{bad_resp, good_resp};

use brewdrivers::model::{Device, RTU};

#[derive(Resource, serde::Deserialize)]
#[resource(update, enact)]
pub struct RTUResource;

/// Calls `update` on the RTU stored in the state, then returns it
/// This is equivalent to calling update on every device, and also returning
/// the extra RTU metadata like name and IP
#[endpoint(uri = "update", method = "Method::GET", params = false, body = false)]
async fn update(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    info!("Current state: {:#?}", rtu_state.inner.lock().await);
    match rtu_state.update().await {
        Ok(_) => good_resp(rtu_state.inner.lock().await.clone(), StatusCode::OK),
        Err(e) => bad_resp(
            RequestError::InstrumentError(e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Enacts the RTU posted, then returns Ok
/// 
/// This compares the incoming RTU configuration with the current one stored in the 
/// web server state. It will create a collection of `Device`s only from the devices in the 
/// incoming config that have different states from the current config. This ensures that we write 
/// the minimum amount of data to the serial devices. Connecting to the devices takes 20-100 ms, so 
/// we save *a lot* of time by doing this.
#[endpoint(uri = "enact", method = "Method::POST", params = false, body = true)]
async fn enact(state: &mut State, mut incoming: RTU) -> Response {
    let mut devs_to_enact: Vec<&mut Device> = Vec::new();

    let rtu_state = RTUState::borrow_from(&state);
    let mut current = rtu_state.inner.lock().await;
    
    for in_dev in incoming.devices.iter_mut() {
        // If we can find a device in the saved state that matches the incoming device
        if let Some(current_dev) = current.device(&in_dev.id) {
            // and the states are the same
            if current_dev.state == in_dev.state {
                // Don't add the device
                continue;
            }
        }
        // if the states are different or a new device is posted, update it
        devs_to_enact.push(in_dev);
    }


    for dev in devs_to_enact.iter_mut() {
        log::trace!("Enacting device `{}`", dev.name);
        if let Err(e) = dev.enact().await {
            return bad_resp(RequestError::InstrumentError(e), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    *current = incoming;
    Response::no_content()
}

#[cfg(test)]
mod tests {
    use super::*;

    use brewdrivers::state::BinaryState;
    use env_logger::Env;
    use gotham::test::{TestResponse, TestServer};
    use hyper::{Body, StatusCode};
    use log::*;

    use brewdrivers::model::Device;
    use tokio_test::assert_err;

    use crate::error::ErrorJson;
    use crate::router;
    use crate::tests::{addr, get, post, resp_to_string};

    #[test]
    fn test_update_rtu() {
        let resp = get("rtu/update");
        let content = resp_to_string(resp);
        let rtu: RTU = serde_json::from_str(&content).expect("Couldn't deserialize rtu");
        assert!(rtu.devices.len() > 0);
        for dev in &rtu.devices {
            assert!(
                dev.state.relay_state.is_some() || dev.state.pv.is_some() || dev.state.sv.is_some()
            );
        }
    }

    #[test]
    fn test_enact_rtu() {
        // First, get the RTU with fresh values
        let content = resp_to_string(get("rtu/update"));
        let mut rtu: RTU = serde_json::from_str(&content).expect("Couldn't deserialize rtu");
        assert!(rtu.devices.len() > 0);

        // If we can find this device, manually change it in the RTU config
        if let Some(mut device) = rtu.device("wsrelay0") {
            device.state.relay_state = Some(BinaryState::On);
        }

        // Post the changed RTU config
        let resp = post("rtu/enact", rtu);
        // Assert we got a good response, no content
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let content2 = resp_to_string(get("rtu/update"));
        let mut rtu2: RTU = serde_json::from_str(&content2).expect("Couldn't deserialize rtu");
        match rtu2.device("wsrelay0") {
            Some(device) => {
                assert_eq!(device.state.relay_state, Some(BinaryState::On));
                device.state.relay_state = Some(BinaryState::Off);
            }
            None => assert!(false, "Device wsrelay0 not found"),
        }

        // Set the relay off again
        post("rtu/enact", rtu2);
    }
}
