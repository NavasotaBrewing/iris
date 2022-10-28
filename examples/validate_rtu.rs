// Here's an example of validating the rtu_conf.yaml file using RTU::generate()

use nbc_iris::model::*;

fn main() {
    match RTU::generate(Some("examples/example_configuration.yaml")) {
        Ok(_) => println!("RTU parsed successfully and passed all validators!"),
        Err(e) => panic!("Error: {}", e)
    }
}
