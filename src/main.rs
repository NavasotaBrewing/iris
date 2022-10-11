pub mod model;
#[cfg(feature = "web")]
pub mod api;

use model::RTU;

#[cfg(feature = "web")]
#[tokio::main]
async fn main() {
    println!("About to start the web server");
    api::run().await;
}