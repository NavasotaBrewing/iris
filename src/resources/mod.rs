pub mod device;
pub mod rtu;

pub use device::DeviceResource;
use gotham::mime;
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
        Some(mime::APPLICATION_JSON)
    )
}

fn bad_resp<E: std::error::Error>(error: E, status: StatusCode) -> Response {
    Response::new(
        status,
        serde_json::to_string(&ErrorJson::from(error)).unwrap(),
        Some(mime::APPLICATION_JSON)
    )
}
