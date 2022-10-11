pub mod model;

use model::{Device, RTU};

use crate::model::{Driver, State};

fn main() {
    let rtu = RTU::generate(None).expect("Error generating RTU");
    println!("{:#?}", rtu);
}
