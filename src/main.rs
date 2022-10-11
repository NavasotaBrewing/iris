pub mod model;

use model::RTU;

fn main() {
    let rtu = RTU::generate(None).expect("Error generating RTU");
    println!("{:#?}", rtu);
}
