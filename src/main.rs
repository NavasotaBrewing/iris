//! Execute the Iris server
use log::info;
use env_logger::Env;
use gotham::{pipeline::{single_pipeline, new_pipeline}, router::build_router};
use gotham_restful::{*, cors::*};
use hyper::header::CONTENT_TYPE;

/// Same as in lib.rs
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

mod resources;

pub fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS web server");

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: Headers::List(vec![CONTENT_TYPE]),
		max_age: 0,
		credentials: true
	};

	let (chain, pipelines) = single_pipeline(new_pipeline().add(cors).build());

	gotham::start(
		"127.0.0.1:7878",
		build_router(chain, pipelines, |route| {
			route.resource::<resources::DeviceResource>("device");
		})
	)
	.expect("Failed to start gotham");
}