use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::gpio;

#[cfg(feature = "raspberry-pi")]
pub mod raspberry_pi;

pub trait GpioImplementor {
    // TODO: allow specifying pullup/pulldown in new_input_pin
    fn new_input_pin(&mut self, pin: u8) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError>;
    fn new_output_pin(&mut self, pin: u8) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError>;
}

pub trait InputPinImplementor: Send + Sync {
    fn read(&self) -> gpio::LogicLevel;
}

impl Debug for dyn InputPinImplementor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputPinImplementor")
            .finish_non_exhaustive()
    }
}

pub trait OutputPinImplementor: Send + Sync {
    fn write(&self, level: gpio::LogicLevel);
}

impl Debug for dyn OutputPinImplementor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputPinImplementor")
            .finish_non_exhaustive()
    }
}
