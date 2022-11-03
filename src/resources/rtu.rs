use gotham::state::State;
use gotham::prelude::*;
use gotham_restful::*;
use hyper::{Method, StatusCode};
use log::info;
use crate::RTUState;

use crate::error::RequestError;
use crate::resp::{good_resp, bad_resp};

use brewdrivers::model::RTU;

#[derive(Resource, serde::Deserialize)]
#[resource(update, enact)]
pub struct RTUResource;


/// Calls `update` on the RTU stored in the state, then returns it
/// This is equivalent to calling update on every device, and also returning
/// the extra RTU metadata like name and IP
#[endpoint(
	uri = "update",
	method = "Method::GET",
	params = false,
	body = false
)]
async fn update(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    match rtu_state.update().await {
        Ok(_) => good_resp(rtu_state.inner.lock().await.clone(), StatusCode::OK),
        Err(e) => bad_resp(RequestError::InstrumentError(e), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Enacts the RTU posted, then returns Ok
#[endpoint(
    uri = "enact",
    method = "Method::POST",
    params = false,
    body = true
)]
async fn enact(state: &mut State, mut body: RTU) -> Response {
    info!("About to update state of RTU `{}`", body.id);
    body.enact().await.unwrap();
    let rtu_state = RTUState::borrow_from(&state);
    *rtu_state.inner.lock().await = body;
    // good_resp(rtu_state.inner.lock().await.clone())
    Response::no_content()
}

