//! Response helper functions

use gotham_restful::Response;
use hyper::StatusCode;
use serde::Serialize;
use crate::error::{RequestError, ErrorJson};

pub(crate) fn good_resp<T: serde::Serialize>(data: T, status: StatusCode) -> Response {
    Response::json(
        status,
        serde_json::to_string(&data).unwrap()
    )
}

pub(crate) fn bad_resp<E: std::error::Error>(error: E, status: StatusCode) -> Response {
    Response::json(
        status,
        serde_json::to_string(&ErrorJson::from(error)).unwrap()
    )
}
