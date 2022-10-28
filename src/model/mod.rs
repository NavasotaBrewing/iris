//! This package contains a representation of an RTU and it's state.
//! 
//! It provides methods for:
//!     
//! 1. Updating the RTU - polling the hardware and updating the values in the RTU state data to match the hardware
//! 2. Enacting the RTU - setting the physical controllers' state to match the RTU state data.
//! 
//! Note that the word "RTU" refers to two things. First, it refers to the physical box on the brewing rig containing a computer and controllers, used 
//! to control devices like valves. Second, within the context of this crate and [RTU](crate::model::RTU), it refers to the digital representation of that RTU through a state object.

mod device;
mod rtu;

pub use device::Device;
pub use rtu::RTU;
pub use rtu::RTUError;
