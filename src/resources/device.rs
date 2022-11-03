use gotham::{
    prelude::StaticResponseExtender,
    state::{FromState, State, StateData},
};
use gotham_restful::*;
use hyper::{Method, StatusCode, Body};
use serde::Deserialize;

use crate::error::RequestError;
use crate::resp::{bad_resp, good_resp};
use crate::RTUState;

use brewdrivers::model::{Device, RTU};

#[derive(Resource, serde::Deserialize)]
#[resource(update_device, enact_device)]
pub struct DeviceResource;

#[derive(Debug, Deserialize, Clone, StateData, StaticResponseExtender)]
pub struct DeviceID {
    id: String,
}

#[endpoint(
    uri = "update/:id",
    method = "Method::GET",
    params = false,
    body = false
)]
pub async fn update_device(id: DeviceID, state: &mut State) -> Response {
    log::trace!("Recieved device id `{}`, about to update", id.id);
    let rtu_state = RTUState::borrow_from(&state);
    let mut rtu = rtu_state.inner.lock().await;

    match rtu.device(&id.id).as_mut() {
        Some(device) => {
            match device.update().await {
                Ok(_) => good_resp(device, StatusCode::OK),
                Err(e) => bad_resp(RequestError::InstrumentError(e), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        None => bad_resp(RequestError::DeviceNotFound(id.id), StatusCode::NOT_FOUND),
    }
}

#[endpoint(
    uri = "enact/:id",
    method = "Method::POST",
    params = false,
    body = true
)]
pub async fn enact_device(id: DeviceID, state: &mut State, mut body: Device) -> Response {
    log::trace!("Recieved device id `{}`, with body: {:?}", id.id, body);
    let rtu_state = RTUState::borrow_from(&state);
    let mut rtu = rtu_state.inner.lock().await;

    match rtu.device(&id.id).as_mut() {
        Some(device) => {
            *device = &mut body;
            device.enact().await.unwrap();
            good_resp(device, StatusCode::OK)
        }
        None => bad_resp(RequestError::DeviceNotFound(id.id), StatusCode::NOT_FOUND),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use brewdrivers::state::BinaryState;
    use env_logger::Env;
    use gotham::test::{TestResponse, TestServer};
    use hyper::{StatusCode, Body};
    use log::*;

    use brewdrivers::model::Device;
    use tokio_test::assert_err;

    use crate::error::ErrorJson;
    use crate::router;
    use crate::tests::{addr, get, post, resp_to_string};

    #[test]
    fn test_update_omega() {
        let response = get("device/update/omega1");

        assert_eq!(response.status(), StatusCode::OK);
        let content = resp_to_string(response);
        let dev: Device = serde_json::from_str(&content).unwrap();
        info!("device: {:?}", dev);
        assert_eq!(dev.id, "omega1");
        assert!(dev.state.pv.is_some());
        assert!(dev.state.sv.is_some());
    }

    #[test]
    fn test_update_str1() {
        let response = get("device/update/relay0");

        assert_eq!(response.status(), StatusCode::OK);
        let content = resp_to_string(response);
        let dev: Device = serde_json::from_str(&content).unwrap();
        info!("device: {:?}", dev);
        assert_eq!(dev.id, "relay0");
        assert!(dev.state.relay_state.is_some());
    }

    #[test]
    fn test_update_wavesharev2() {
        let response = get("device/update/wsrelay0");

        assert_eq!(response.status(), StatusCode::OK);
        let content = resp_to_string(response);
        let dev: Device = serde_json::from_str(&content).unwrap();
        info!("device: {:?}", dev);
        assert_eq!(dev.id, "wsrelay0");
        assert!(dev.state.relay_state.is_some());
    }

    #[test]
    fn test_update_device_not_found() {
        let response = get("device/update/definitely_not_a_device_id");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let content = resp_to_string(response);
        let resp_json: ErrorJson = serde_json::from_str(&content).unwrap();
        assert!(resp_json.message().contains("not found"));
    }

    #[ignore = "This makes the RTU configuration invalid and not usable for other programs"]
    #[test]
    fn test_instrument_error() {
        let response = get("device/update/wserror");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let content = resp_to_string(response);
        let resp_json: ErrorJson = serde_json::from_str(&content).unwrap();
        assert!(resp_json.message().contains("Instrument error"));
    }

    #[test]
    fn test_enact_device() {
        let resp = get("device/update/wsrelay0");
        let mut device: Device = serde_json::from_str(
            &resp_to_string(resp)
        ).unwrap();
        assert!(device.state.relay_state.is_some());
        device.state.relay_state = Some(BinaryState::On);

        let resp2 = post("device/enact/wsrelay0", device);
        assert_eq!(resp2.status(), StatusCode::OK);
        
        let resp3 = get("device/update/wsrelay0");
        let mut device2: Device = serde_json::from_str(
            &resp_to_string(resp3)
        ).unwrap();
        assert_eq!(device2.state.relay_state, Some(BinaryState::On));

        device2.state.relay_state = Some(BinaryState::Off);
        post("device/enact/wsrelay0", device2);
    }
}
