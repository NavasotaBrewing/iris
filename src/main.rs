pub mod model;

use model::{Device, RTU};

use crate::model::{Driver, State};

fn main() {
    // let dev1: Device = Device {
    //     driver: Driver::Omega,
    //     name: String::from("Thermometer 1"),
    //     id: String::from("thermometer-1"),
    //     state: State::Off,
    //     addr: 3,
    //     controller_addr: 1,
    //     pv: None,
    //     sv: None
    // };
    // let dev2: Device = Device {
    //     driver: Driver::Omega,
    //     name: String::from("Thermometer 2"),
    //     id: String::from("thermometer-2"),
    //     state: State::Off,
    //     addr: 4,
    //     controller_addr: 1,
    //     pv: None,
    //     sv: None
    // };
    // let dev3: Device = Device {
    //     driver: Driver::Waveshare,
    //     name: String::from("Relay 1"),
    //     id: String::from("relay-1"),
    //     state: State::Off,
    //     addr: 1,
    //     controller_addr: 4,
    //     pv: None,
    //     sv: None
    // };

    // let rtu: RTU = RTU {
    //     name: String::from("Hot side"),
    //     id: String::from("main-rtu-id"),
    //     devices: vec![dev1, dev2, dev3]
    // };


    // println!("{:#?}", rtu);
    // println!("{}", serde_json::to_string_pretty(&rtu).unwrap());
    // println!("{}", serde_yaml::to_string(&rtu).unwrap());
}
