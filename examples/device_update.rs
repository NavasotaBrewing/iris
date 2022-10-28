use brewdrivers::controllers::*;
use nbc_iris::model::Device;

#[tokio::main]
async fn main() {
    // Manually build a device
    // Usually, this is deserialized from a yaml configuration file
    let mut dev = Device {
        id: String::from("my-dev"),
        name: String::from("My Dev"),
        port: String::from("/dev/ttyUSB0"),
        addr: 0,
        controller: Controller::CN7500,
        controller_addr: 22,
        state: AnyState::BinaryState(BinaryState::On),
        pv: None,
        sv: None,
    };

    assert!(dev.pv.is_none());

    // Update it and watch the values populate
    dev.update().await.unwrap();
    
    assert!(dev.pv.is_some());
    println!("Device PV: {}", dev.pv.unwrap());
}
