//! Execute the Iris server
#![feature(trivial_bounds)]
#![allow(dead_code, unused_imports)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use env_logger::Env;
use gotham::{
    middleware::state::StateMiddleware,
    pipeline::{new_pipeline, single_pipeline},
    router::{build_router, Router},
    state::StateData,
};
use gotham_restful::{cors::*, *};
use hyper::header::CONTENT_TYPE;
use log::info;

use brewdrivers::{
    drivers::InstrumentError,
    model::{Device, RTU},
};

/// Same as in lib.rs
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

mod error;
mod resources;
mod resp;

#[derive(Clone, StateData)]
struct RTUState {
    inner: Arc<Mutex<RTU>>,
}

impl std::panic::RefUnwindSafe for RTUState {}

impl RTUState {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                // TODO: Error handling
                RTU::generate(None).unwrap(),
            )),
        }
    }

    async fn update(&self) -> Result<(), InstrumentError> {
        let mut r = self.inner.lock().await;
        (*r).update().await?;
        Ok(())
    }

    /// This function is extremely costly and you shouldn't use it. Instead, 
    /// call `Device::enact()` on only the devices that need it
    #[deprecated = "This is too costly, use `brewdrivers::model::Device::enact()` instead"]
    async fn enact(&self) -> Result<(), InstrumentError> {
        let mut r = self.inner.lock().await;
        (*r).enact().await
    }
}

fn router() -> Router {
    let cors = CorsConfig {
        origin: Origin::Copy,
        headers: Headers::List(vec![CONTENT_TYPE]),
        max_age: 0,
        credentials: true,
    };

    let rtu = RTUState::new();

    let middleware = StateMiddleware::new(rtu);

    let (chain, pipelines) = single_pipeline(new_pipeline().add(cors).add(middleware).build());

    build_router(chain, pipelines, |route| {
        route.resource::<resources::DeviceResource>("device");
        route.resource::<resources::RTUResource>("rtu");
    })
}

pub fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS web server");

    gotham::start("127.0.0.1:7878", router()).expect("Failed to start gotham");
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use env_logger::Env;
    use gotham::{test::{TestResponse, TestServer}, mime};
    use hyper::Body;

    // Enable env_logger in tests
    #[ctor::ctor]
    fn init() {
        env_logger::Builder::from_env(Env::default().default_filter_or("nbc_iris=warn")).init();
    }

    // The following methods are used as helper methods in
    // the other modules' tests
    pub fn addr(uri: &str) -> String {
        format!("http://localhost:7878/{}", uri)
    }

    pub fn resp_to_string(resp: TestResponse) -> String {
        String::from_utf8(resp.read_body().unwrap()).unwrap()
    }

    pub(crate) fn get(uri: &str) -> TestResponse {
        let test_server = TestServer::new(router()).unwrap();
        test_server
            .client()
            .get(addr(uri))
            .perform()
            .unwrap()
    }

    pub(crate) fn post<T: Serialize>(uri: &str, body: T) -> TestResponse {
        let test_server = TestServer::new(router()).unwrap();
        test_server
            .client()
            .post(addr(uri), serde_json::to_string(&body).unwrap(), mime::APPLICATION_JSON)
            .perform()
            .unwrap()
    }
}
