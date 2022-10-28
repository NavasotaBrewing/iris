use thiserror::Error;

#[derive(Error, Debug)]
pub enum RTUError {
    #[error("Configuration file not found at /etc/NavasotaBrewing/rtu_conf.yaml")]
    FileNotFound,

    #[error("IO error: {0}")]
    IOError(std::io::Error),

    #[error("Permission error, cannot access /etc/NavasotaBrewing/rtu_conf.yaml")]
    PermissionError,

    #[error("Serde parse error: {0}")]
    SerdeParseError(serde_yaml::Error),

    #[error("Validation Error for (k,v) = (`{key}`,`{value}`): {msg}")]
    ValidationError {
        key: String,
        value: String,
        msg: String,
    }
}

impl RTUError {
    /// Constructs an `RTUError::ValidationError`
    pub fn validation_error(key_value: (&str, &str), msg: &str) -> RTUError {
        return RTUError::ValidationError {
            key: key_value.0.to_string(),
            value: key_value.1.to_string(),
            msg: msg.to_string(),
        };
    }
}
