use gotham_restful::*;
use hyper::StatusCode;

use brewdrivers::model::Device;

#[derive(Resource, serde::Deserialize)]
#[resource(read_all, read, update)]
pub struct DeviceResource(Device);

fn good_resp<T: serde::Serialize>(data: T) -> Response {
    Response::new(
        StatusCode::OK,
        serde_json::to_string(&data).unwrap(),
        None
    )
}

/// Reads all devices configured
#[read_all]
fn read_all() -> Response {
    // let rtu = RTU::generate(None).unwrap();
	// Response::new(StatusCode::OK, serde_json::to_string(&rtu.devices).unwrap(), None)
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