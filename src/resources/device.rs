use gotham::{state::{StateData, FromState, State}, prelude::StaticResponseExtender};
use gotham_restful::*;
use hyper::{StatusCode, Method};
use serde::{Deserialize};
use crate::RTUState;

use super::{good_resp, bad_resp};

use brewdrivers::model::{Device, RTU};

#[derive(Resource, serde::Deserialize)]
#[resource(update_device)]
pub struct DeviceResource;

#[derive(Deserialize, Clone, StateData, StaticResponseExtender)]
pub struct DeviceID(String);

#[endpoint(
    uri = "update/:id",
    method = "Method::GET",
    params = false,
    body = false
)]
pub async fn update_device(id: DeviceID, state: &mut State) -> Response {
    // log::info!("Got id: `{}`", id.0);
    // let rtu_state = RTUState::borrow_from(&state);
    // let mut rtu = rtu_state.inner.lock().await;
    // if let Some(device) = rtu.device(&id.0).as_mut() {
    //     log::info!("got device: {:?}", device);
    // }
    Response::no_content()
}
