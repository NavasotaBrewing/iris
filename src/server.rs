//! The Iris server, build on Warp
//! 
//! This does not get compiled if using the crate as a library, only as an executable

use log::{error, info, trace, debug};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::model::RTU;
use warp::http::Response;
use warp::{hyper::Method, Filter};

/// A basic error response, to serialize to json
#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    msg: String,
}

/// Creates a JSON response object with an error message
fn json_error_resp(msg: String) -> Result<Response<String>, warp::http::Error> {
    let resp_str = serde_json::to_string(&ErrorResponse { msg }).unwrap();
    // It's VERY important to set that header
    Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .status(500)
        .body(resp_str)
}

/// Creates a JSON response object with any arbitrary serializable data
fn json_response<T: Serialize>(value: T) -> Result<Response<String>, warp::http::Error> {
    let resp_str = serde_json::to_string(&value).unwrap();
    // It's VERY important to set that header
    Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .status(200)
        .body(resp_str)
}

/// Creates a warp server and runs it
pub async fn run() {
    // Log setup
    let incoming_log = warp::log::custom(|info| {
        info!("");
        info!("=== New Request ===");
        info!("remote addr: {:?}", info.remote_addr());
        info!("method: {:?}", info.method());
        info!("path: {:?}", info.path());
        // info!("version: {:#?}", info.version());
        info!("status: {:?}", info.status());
        // info!("referer: {:#?}", info.referer());
        // info!("user_agent: {:#?}", info.user_agent());
        // info!("elapsed: {:#?}", info.elapsed());
        info!("host: {:?}", info.host());
        // info!("request_headers: {:#?}", info.request_headers());
    });

    // CORS setup
    // Please don't mess with this
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(&[Method::GET, Method::POST, Method::OPTIONS]);
    
    // Routes
    // Responds to /running with a payload containing true, just for testing
    let running = warp::path("running")
        .map(|| r#"{"running":"true"}"#)
        .with(&incoming_log)
        .with(&cors);

    let generate_rtu_route = warp::path("generate")
        .and_then(generate_rtu)
        .with(&incoming_log)
        .with(&cors);

    let update_rtu_route = warp::path("update")
        .and(warp::body::json())
        .and_then(update_rtu)
        .with(&incoming_log)
        .with(&cors);

    let enact_rtu_route = warp::path("enact")
        .and(warp::body::json())
        .and_then(enact_rtu)
        .with(&incoming_log)
        .with(&cors);

    let routes = running
        .or(generate_rtu_route)
        .or(update_rtu_route)
        .or(enact_rtu_route);


    match RTU::generate(None) {
        Ok(rtu) => {
            info!("RTU configuration serialized successfully");
            info!("{} device(s) configured", rtu.devices.len());
            warp::serve(routes).run(([0, 0, 0, 0], 3012)).await;
        }
        Err(e) => {
            error!("RTU configuration couldn't not be deserialized");
            error!("{}", e);
            error!("Aborting. Fix your configuration file.");
            std::process::exit(1);
        }
    };
    
}

/// Receives the RTU model and updates the hardware to match, aka Write mode
async fn enact_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    trace!("RTU recieved payload, enacting changes");
    match RTU::enact(&mut rtu).await {
        Ok(_) => return Ok(json_response(&rtu)),
        Err(e) => return Ok(json_error_resp(format!("error: {}", e))),
    };
}

/// Receives the RTU model and updates it to match the hardware, aka Read mode
async fn update_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    trace!("RTU recieved payload, updating and sending it back");
    match RTU::update(&mut rtu).await {
        Ok(_) => {
            debug!("RTU String: {:?}", json_response(&rtu));
            Ok(json_response(&rtu))
        },
        Err(e) => Ok(json_error_resp(format!("error: {}", e))),
    }
}

/// Generates the RTU model from the configuration file
async fn generate_rtu() -> Result<impl warp::Reply, Infallible> {
    let rtu = match RTU::generate(None) {
        Ok(rtu) => rtu,
        Err(e) => {
            error!("Couldn't generate RTU from config file: {}", e);
            std::process::exit(1);
        }
    };

    match serde_json::to_string(&rtu) {
        Ok(rtu) => {
            debug!("RTU String: {}", rtu);
            Ok(rtu)
        },
        Err(e) => {
            error!("Couldn't serialize RTU: {}", e);
            std::process::exit(1);
        }
    }
}
