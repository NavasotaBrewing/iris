use gotham::state::State;
use gotham::prelude::*;
use gotham_restful::*;
use hyper::Method;
use log::info;
use crate::RTUState;

use super::good_resp;

use brewdrivers::model::RTU;

#[derive(Resource, serde::Deserialize)]
#[resource(read_all, read, update)]
pub struct RTUResource;


/// This just returns the RTU stored in the state
#[endpoint(
    uri = "generate",
    method = "Method::GET",
    params = false,
    body = false
)]
async fn read_all(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    let rtu = rtu_state.inner.lock().await;
    good_resp(rtu.clone())
}

/// Reads the RTU (match RTU values to controllers)
#[endpoint(
	uri = "read",
	method = "Method::GET",
	params = false,
	body = false
)]
async fn read(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    // TODO: Error handling
    rtu_state.update().await.unwrap();
    let rtu = rtu_state.inner.lock().await;
    good_resp(rtu.clone())
}


/// Updates an RTU (write state to controllers)
#[endpoint(
    uri = "update",
    method = "Method::POST",
    params = false,
    body = true
)]
async fn update(state: &mut State, mut body: RTU) -> Response {
    info!("About to update state of RTU `{}`", body.id);
    body.enact().await.unwrap();
    let rtu_state = RTUState::borrow_from(&state);
    *rtu_state.inner.lock().await = body;
    good_resp(rtu_state.inner.lock().await.clone())
}
