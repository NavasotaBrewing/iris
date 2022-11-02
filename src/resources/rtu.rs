use gotham::state::State;
use gotham::prelude::*;
use gotham_restful::*;
use hyper::{Method, StatusCode};
use log::info;
use crate::RTUState;

use super::{good_resp, bad_resp};

use brewdrivers::model::RTU;

#[derive(Resource, serde::Deserialize)]
#[resource(generate, update, enact)]
pub struct RTUResource;


/// This just returns the RTU stored in the state
#[endpoint(
    uri = "generate",
    method = "Method::GET",
    params = false,
    body = false
)]
async fn generate(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    let rtu = rtu_state.inner.lock().await;
    good_resp(rtu.clone(), StatusCode::OK)
}

/// Calls `update` on the RTU stored in the state, then returns it
#[endpoint(
	uri = "update",
	method = "Method::GET",
	params = false,
	body = false
)]
async fn update(state: &mut State) -> Response {
    let rtu_state = RTUState::borrow_from(&state);
    // TODO: Error handling
    match rtu_state.update().await {
        Ok(_) => {
            let rtu = rtu_state.inner.lock().await;
            good_resp(rtu.clone(), StatusCode::OK)
        },
        Err(e) => bad_resp(e, StatusCode::INTERNAL_SERVER_ERROR)
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
