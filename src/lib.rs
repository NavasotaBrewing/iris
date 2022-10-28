//! This crate is the networking component the Navasota Brewing Company's "Brewery Control System". 
//! 
//! This docs contain only information about the code. For information about it's usage and what this crate
//! does, see the [readme](https://github.com/NavasotaBrewing/iris) or the [organization documentation](https://github.com/NavasotaBrewing/documentation)
//!
//! This crate is both a library and executable. It can be installed and run through cargo, or used by another crate.

pub mod model;

/// Location of the configuration file
/// 
/// This is static across all NBC packages. It should remain constant.
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";
