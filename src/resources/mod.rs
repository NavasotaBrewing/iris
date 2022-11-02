mod device;
mod rtu;

pub use device::DeviceResource;
pub use rtu::RTUResource;

use serde::Serialize;
use gotham_restful::Response;
use hyper::StatusCode;

/// A wrapper for an error message
#[derive(Serialize)]
struct ErrorJson {
    error: String
}

impl ErrorJson {
    /// Wraps a given error in this type
    fn from<E: std::error::Error>(error: E) -> Self {
        ErrorJson {
            error: format!("{}", error)
        }
    }
}

fn good_resp<T: serde::Serialize>(data: T, status: StatusCode) -> Response {
    Response::new(
        status,
        serde_json::to_string(&data).unwrap(),
        None
    )
}

fn bad_resp<E: std::error::Error>(error: E, status: StatusCode) -> Response {
    Response::new(
        status,
        serde_json::to_string(&ErrorJson::from(error)).unwrap(),
        None
    )
}
