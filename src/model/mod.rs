use serde::{Serialize, Deserialize};

mod device;
mod rtu;

pub use device::Device;
pub use rtu::RTU;

/// When a model is being passed back and forth from the web interface to here, it
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    
    #[test]
    fn test_mode_serialize() {
        assert_eq!(serde_json::to_string(&Mode::Write).unwrap(), "\"Write\"");
        assert_eq!(serde_json::to_string(&Mode::Read).unwrap(), "\"Read\"");
        // they are case sensitive!
        assert_ne!(serde_json::to_string(&Mode::Write).unwrap(), "\"write\"");
        assert_ne!(serde_json::to_string(&Mode::Read).unwrap(), "\"read\"");

        assert_eq!(Mode::default(), Mode::Read);
    }

}