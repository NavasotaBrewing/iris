use std::convert::Infallible;

use crate::model::{Mode, RTU};
use warp::{hyper::Method, Filter};

/// Creates a warp server and runs it
pub async fn run() {
    let incoming_log = warp::log::custom(|info| {
        eprintln!();
        eprintln!("remote addr: {:#?}", info.remote_addr());
        eprintln!("method: {:#?}", info.method());
        eprintln!("path: {:#?}", info.path());
        eprintln!("version: {:#?}", info.version());
        eprintln!("status: {:#?}", info.status());
        eprintln!("referer: {:#?}", info.referer());
        eprintln!("user_agent: {:#?}", info.user_agent());
        eprintln!("elapsed: {:#?}", info.elapsed());
        eprintln!("host: {:#?}", info.host());
        eprintln!("request_headers: {:#?}", info.request_headers());
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Access-Control-Allow-Origin", "Origin", "Accept", "X-Requested-With", "Content-Type"])
        .allow_methods(&[Method::GET, Method::POST]);


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

    warp::serve(routes).run(([0, 0, 0, 0], 3012)).await;
}


/// Receives the RTU model and updates the hardware to match, aka Write mode
async fn enact_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    println!("RTU recieved model, enacting changes");
    RTU::update(&mut rtu, &Mode::Write).await;
    // It's VERY important to set that header
    Ok(
        warp::reply::with_header(
            serde_json::to_string(&rtu).expect("Couldn't serialize model"),
            "Access-Control-Allow-Origin",
            "*"
        )
    )
}

/// Receives the RTU model and updates it to match the hardware, aka Read mode
async fn update_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    println!("RTU recieved model, updating and sending it back");
    RTU::update(&mut rtu, &Mode::Read).await;
    // Ok(serde_json::to_string(&rtu).expect("Couldn't serialize model"))
    Ok(
        warp::reply::with_header(
            serde_json::to_string(&rtu).expect("Couldn't serialize model"),
            "Access-Control-Allow-Origin", "*"
        )
    )
}

/// Generates the RTU model from the configuration file
async fn generate_rtu() -> Result<impl warp::Reply, Infallible> {
    println!("Got a ping, generating RTU model");
    let rtu = RTU::generate(None).expect("Couldn't generate RTU from configuration file");
    Ok(serde_json::to_string(&rtu).expect("Couldn't serialize rtu"))
}
