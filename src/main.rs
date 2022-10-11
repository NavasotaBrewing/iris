pub mod model;

// Only compile that module if we want the web server
// This is good for the CLI because we can use the RTU generation
// code without compiling warp and tokio
#[cfg(feature = "web")]
pub mod server;

#[cfg(feature = "web")]
#[tokio::main]
async fn main() {
    println!("About to start the web server");
    server::run().await;
}