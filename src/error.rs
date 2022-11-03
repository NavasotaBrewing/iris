use brewdrivers::drivers::InstrumentError;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// An error type meant to go into a JSON object for returning from routes
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Device with ID `{0}` not found on this RTU")]
    DeviceNotFound(String),
    #[error("Instrument error: {0}")]
    InstrumentError(InstrumentError)
}

/// A wrapper for an error message
#[derive(Serialize, Deserialize)]
pub struct ErrorJson {
    error: String
}

impl ErrorJson {
    /// Wraps a given error in this type
    pub fn from<E: std::error::Error>(error: E) -> Self {
        ErrorJson {
            error: format!("{}", error)
        }
    }

    pub fn message(&self) -> &String {
        &self.error
    }
}