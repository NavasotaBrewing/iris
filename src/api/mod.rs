use std::convert::Infallible;

use warp::Filter;
use crate::model::{RTU, Mode};

pub async fn run() {
    let running = warp::path("running").map(|| r#"{"running":"true"}"# );

    let generate_rtu_route = warp::path("generate")
        .and_then(generate_rtu);
    
    let update_rtu_route = warp::path("update")
        .and(warp::body::json())
        .and_then(update_rtu);

    let enact_rtu_route = warp::path("enact")
        .and(warp::body::json())
        .and_then(enact_rtu);

    let routes = running
        .or(generate_rtu_route)
        .or(update_rtu_route)
        .or(enact_rtu_route);
    warp::serve(routes).run(([0,0,0,0], 3012)).await;
}

async fn enact_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    println!("RTU recieved model, enacting changes");
    let updated = RTU::update(&mut rtu, &Mode::Write).await;
    Ok(serde_json::to_string(&updated).expect("Couldn't serialize model"))
}

async fn update_rtu(mut rtu: RTU) -> Result<impl warp::Reply, Infallible> {
    println!("RTU recieved model, updating and sending it back");
    let updated = RTU::update(&mut rtu, &Mode::Read).await;
    Ok(serde_json::to_string(&updated).expect("Couldn't serialize model"))
}

async fn generate_rtu() -> Result<impl warp::Reply, Infallible> {
    println!("Got a ping, generating RTU model");
    let rtu = RTU::generate(None).expect("Couldn't generate RTU from configuration file");
    Ok(serde_json::to_string(&rtu).expect("Couldn't serialize rtu"))
}