//! Execute the Iris server
use std::sync::Arc;
use tokio::sync::Mutex;

use log::info;
use env_logger::Env;
use gotham::{pipeline::{single_pipeline, new_pipeline}, router::build_router, state::StateData, middleware::state::StateMiddleware};
use gotham_restful::{*, cors::*};
use hyper::header::CONTENT_TYPE;

use brewdrivers::{model::RTU, drivers::InstrumentError};

/// Same as in lib.rs
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

mod resources;

#[derive(Clone, StateData)]
struct RTUState {
	inner: Arc<Mutex<RTU>>
}

impl std::panic::RefUnwindSafe for RTUState {}

impl RTUState {
	fn new() -> Self {
		Self {
			inner: Arc::new(Mutex::new(
				// TODO: Error handling
				RTU::generate(None).unwrap()
			))
		}
	}

	async fn update(&self) -> Result<RTU, InstrumentError> {
		let mut r = self.inner.lock().await;
		(*r).update().await?;
		Ok(r.clone())
	}

	async fn enact(&self) -> Result<(), InstrumentError> {
		let mut r = self.inner.lock().await;
		(*r).enact().await
	}
}


pub fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS web server");

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: Headers::List(vec![CONTENT_TYPE]),
		max_age: 0,
		credentials: true
	};

	let rtu = RTUState::new();


	let middleware = StateMiddleware::new(rtu);

	let (chain, pipelines) = single_pipeline(
		new_pipeline()
			.add(cors)
			.add(middleware)
			.build()
	);

	gotham::start(
		"127.0.0.1:7878",
		build_router(chain, pipelines, |route| {
			route.resource::<resources::DeviceResource>("device");
            route.resource::<resources::RTUResource>("rtu");
		})
	)
	.expect("Failed to start gotham");
}