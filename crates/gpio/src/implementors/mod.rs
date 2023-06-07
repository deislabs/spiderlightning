use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;

use crate::{gpio, Pin, Pull};

#[cfg(feature = "raspberry_pi")]
pub mod raspberry_pi;

/// A GPIO implementation.
/// 
/// This trait is not referred to directly by the linked capability, but is used to construct pin resources implementing the other traits in this module.
pub(crate) trait GpioImplementor {
    /// Constructs an input pin resource.
    fn new_input_pin(
        &mut self,
        pin: u8,
        pull: Option<Pull>,
    ) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError>;
    /// Constructs an output pin resource.
    fn new_output_pin(
        &mut self,
        pin: u8,
        init_level: Option<gpio::LogicLevel>,
    ) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError>;
}

impl dyn GpioImplementor {
    /// Parse the provided configuration string and construct the appropriate pin resource.
    pub(crate) fn parse_pin_config(&mut self, config: &str) -> Result<Pin, gpio::GpioError> {
        let mut config_iter = config.split('/');

        let pin_number = config_iter
            .next()
            .ok_or_else(|| gpio::GpioError::ConfigurationError(String::from("no pin number")))?;
        let pin_number = u8::from_str(pin_number).map_err(|e| {
            gpio::GpioError::ConfigurationError(format!("invalid pin number '{pin_number}': {e}"))
        })?;

        match config_iter.next() {
            Some("input") => {
                let pull = if let Some(pull) = config_iter.next() {
                    Some(match pull {
                        "pullup" => Pull::Up,
                        "pulldown" => Pull::Down,
                        _ => Err(gpio::GpioError::ConfigurationError(format!(
                            "unknown pull setting '{pull}'"
                        )))?,
                    })
                } else {
                    None
                };
                if config_iter.next().is_some() {
                    return Err(gpio::GpioError::ConfigurationError(String::from(
                        "too many fields for input pin",
                    )));
                }
                self.new_input_pin(pin_number, pull).map(Pin::Input)
            }
            Some("output") => {
                let init_level = if let Some(init_level) = config_iter.next() {
                    Some(match init_level {
                        "low" => gpio::LogicLevel::Low,
                        "high" => gpio::LogicLevel::High,
                        _ => Err(gpio::GpioError::ConfigurationError(format!(
                            "unknown initial level '{init_level}'"
                        )))?,
                    })
                } else {
                    None
                };
                if config_iter.next().is_some() {
                    return Err(gpio::GpioError::ConfigurationError(String::from(
                        "too many fields for output pin",
                    )));
                }
                self.new_output_pin(pin_number, init_level).map(Pin::Output)
            }
            Some(unknown_type) => Err(gpio::GpioError::ConfigurationError(format!(
                "unknown pin type '{unknown_type}'"
            ))),
            None => Err(gpio::GpioError::ConfigurationError(String::from(
                "no pin type",
            ))),
        }
    }
}

/// An implementation of an input pin resource.
pub trait InputPinImplementor: Send + Sync {
    /// Read the current logic level to the pin.
    fn read(&self) -> gpio::LogicLevel;
}

impl Debug for dyn InputPinImplementor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputPinImplementor")
            .finish_non_exhaustive()
    }
}


/// An implementation of an output pin resource.
pub trait OutputPinImplementor: Send + Sync {
    /// Write the given logic level to the pin.
    fn write(&self, level: gpio::LogicLevel);
}

impl Debug for dyn OutputPinImplementor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputPinImplementor")
            .finish_non_exhaustive()
    }
}
