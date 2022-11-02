mod device;
mod rtu;

pub use device::DeviceResource;
pub use rtu::RTUResource;

use gotham_restful::Response;
use hyper::StatusCode;

fn good_resp<T: serde::Serialize>(data: T) -> Response {
    Response::new(
        StatusCode::OK,
        serde_json::to_string(&data).unwrap(),
        None
    )
}
