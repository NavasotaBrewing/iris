use gotham::state::{State, FromState};
use gotham_restful::*;
use crate::RTUState;

use super::good_resp;

use brewdrivers::model::Device;

#[derive(Resource, serde::Deserialize)]
#[resource(read_all, read, update)]
pub struct DeviceResource;

/// Reads all devices configured
#[read_all]
fn read_all(state: &mut State) -> Response {
    // let rtu = RTU::generate(None).unwrap();
	// Response::new(StatusCode::OK, serde_json::to_string(&rtu.devices).unwrap(), None)
    let rtu = RTUState::borrow_from(&state);
    log::info!("{:?}", rtu.inner);
    good_resp("todo!()")
}

/// Reads a specific device
#[read]
fn read(_: String) -> Response {
    good_resp("todo!()")
}

/// Updates a device
#[update]
fn update(_: String, _: Device) -> Response {
    good_resp("todo!()")
}