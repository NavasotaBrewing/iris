use std::fs;

use serde::{Serialize, Deserialize};

mod device;
mod rtu;
// mod model;

pub use device::Device;
pub use rtu::RTU;
// pub use model::Model;

/// RTUs store their ID in `/rtu_id`. This function reads that ID.
pub fn get_rtu_id() -> String {
    // TODO: Put this in a better place.
    String::from(fs::read_to_string("/rtu_id").expect("Couldn't read RTU id").trim())
}


/// This represents the state of a device. We have the enum to be explicit
/// rather than just using a bool or 0-1. In the future, this may be expanded
/// to account for step values.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum State {
    On,
    Off
}

impl Default for State {
    /// Defaults to Off
    fn default() -> Self { State::Off }
}


/// When a model is being passed back and forth from `brewkit` to here, it
/// can have one of two modes: Read or Write. If the mode is Write, then this crate
/// will update the device state (through the `brewdrivers` crate). If the mode is Read,
/// this crate will update the model with the current device state (also through `brewdrivers`)
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Mode {
    Write,
    Read
}

impl Default for Mode {
    /// Defaults to Read
    fn default() -> Self { Mode::Read }
}


/// These are the types of devices that the BCS supports. This enum should reflect every
/// driver in `brewdrivers`. This enum is just a marker so the Model knows which driver to use.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Driver {
    /// An STR1XX relay board. They come in STR116 (16-relay) or STR108 (8-relay).
    /// The driver is the same either way.
    STR1,
    /// An OMEGA Engineering PID. We use the CN7500, and haven't yet tested on others.
    // TODO: This needs to be renamed CN7500 to be more specific.
    Omega,
    // The Waveshare relay board, similar in usage to the STR1
    Waveshare
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_state_serialize() {
        assert_eq!(serde_json::to_string(&State::On).unwrap(), "\"On\"");
        assert_eq!(serde_json::to_string(&State::Off).unwrap(), "\"Off\"");
        // they are case sensitive!
        assert_ne!(serde_json::to_string(&State::On).unwrap(), "\"on\"");
        assert_ne!(serde_json::to_string(&State::Off).unwrap(), "\"off\"");

        assert_eq!(State::default(), State::Off);
    }
    
    #[test]
    fn test_mode_serialize() {
        assert_eq!(serde_json::to_string(&Mode::Write).unwrap(), "\"Write\"");
        assert_eq!(serde_json::to_string(&Mode::Read).unwrap(), "\"Read\"");
        // they are case sensitive!
        assert_ne!(serde_json::to_string(&Mode::Write).unwrap(), "\"write\"");
        assert_ne!(serde_json::to_string(&Mode::Read).unwrap(), "\"read\"");

        assert_eq!(Mode::default(), Mode::Read);
    }

    #[test]
    fn test_driver_serialize() {
        // These are also case sensitive, take my word for it.
        assert_eq!(serde_json::to_string(&Driver::STR1).unwrap(), "\"STR1\"");
        assert_eq!(serde_json::to_string(&Driver::Omega).unwrap(), "\"Omega\"");
        assert_eq!(serde_json::to_string(&Driver::Waveshare).unwrap(), "\"Waveshare\"");
    }


}