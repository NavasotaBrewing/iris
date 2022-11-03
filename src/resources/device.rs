use gotham::{state::{StateData, FromState, State}, prelude::StaticResponseExtender};
use gotham_restful::*;
use hyper::{StatusCode, Method};
use serde::{Deserialize};

use crate::RTUState;
use crate::resp::{good_resp, bad_resp};
use crate::error::RequestError;

use brewdrivers::model::{Device, RTU};

#[derive(Resource, serde::Deserialize)]
#[resource(update_device)]
pub struct DeviceResource;

#[derive(Debug, Deserialize, Clone, StateData, StaticResponseExtender)]
pub struct DeviceID {
    id: String
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
            device.update().await.unwrap();
            good_resp(device, StatusCode::OK)
        },
        None => bad_resp(RequestError::DeviceNotFound(id.id), StatusCode::NOT_FOUND)
    }
}











#[cfg(test)]
mod tests {
    use super::*;
    
    use env_logger::Env;
	use log::*;
    use gotham::test::{TestServer, TestResponse};
    use hyper::StatusCode;
    
    use brewdrivers::model::Device;
	
    use crate::tests::{addr, resp_to_string};
    use crate::error::ErrorJson;
    use crate::router;


    #[test]
    fn test_update_omega() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get(addr("device/update/omega1"))
            .perform()
            .unwrap();

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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get(addr("device/update/relay0"))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
		let content = resp_to_string(response);
		let dev: Device = serde_json::from_str(&content).unwrap();
		info!("device: {:?}", dev);
		assert_eq!(dev.id, "relay0");
		assert!(dev.state.relay_state.is_some());
    }

	#[test]
    fn test_update_wavesharev2() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get(addr("device/update/wsrelay0"))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
		let content = resp_to_string(response);
		let dev: Device = serde_json::from_str(&content).unwrap();
		info!("device: {:?}", dev);
		assert_eq!(dev.id, "wsrelay0");
		assert!(dev.state.relay_state.is_some());
    }

	#[test]
	fn test_update_device_not_found() {
		let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get(addr("device/update/definitely_not_a_device_id"))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
		let content = resp_to_string(response);
		let resp_json: ErrorJson = serde_json::from_str(&content).unwrap();
		assert!(resp_json.message().contains("not found"));
	}
}