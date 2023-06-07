use std::sync::Mutex;

use rppal::gpio::{Gpio, InputPin, Level, OutputPin};

use super::*;
use crate::{gpio, Pull};

/// A GPIO implementation backed by the [rppal] crate.
pub(crate) struct PiGpioImplementor {
    gpio: Result<Gpio, gpio::GpioError>,
}

impl PiGpioImplementor {
    /// Construct a new [PiGpioImplementor].
    ///
    /// If constructing the underlying [Gpio] object fails, the error will be stored for reporting to the application code.
    pub(crate) fn new() -> Self {
        Self {
            gpio: Gpio::new().map_err(|e| gpio::GpioError::HardwareError(e.to_string())),
        }
    }
}

impl GpioImplementor for PiGpioImplementor {
    fn new_input_pin(
        &mut self,
        pin: u8,
        pull: Option<Pull>,
    ) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?;
        let input_pin = match pull {
            Some(Pull::Up) => pin.into_input_pullup(),
            Some(Pull::Down) => pin.into_input_pulldown(),
            None => pin.into_input(),
        };
        Ok(Arc::new(PiInputPinImplementor { input_pin }))
    }

    fn new_output_pin(
        &mut self,
        pin: u8,
        init_level: Option<gpio::LogicLevel>,
    ) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let mut output_pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?
            .into_output();
        match init_level {
            Some(gpio::LogicLevel::Low) => output_pin.set_low(),
            Some(gpio::LogicLevel::High) => output_pin.set_high(),
            None => (),
        }
        let output_pin = Mutex::new(output_pin);
        Ok(Arc::new(PiOutputPinImplementor { output_pin }))
    }
}

/// An input pin resource backed by [rppal]'s [InputPin].
pub struct PiInputPinImplementor {
    input_pin: InputPin,
}

impl InputPinImplementor for PiInputPinImplementor {
    fn read(&self) -> gpio::LogicLevel {
        match self.input_pin.read() {
            Level::Low => gpio::LogicLevel::Low,
            Level::High => gpio::LogicLevel::High,
        }
    }
}

/// An output pin resource backed by [rppal]'s [OutputPin].
pub struct PiOutputPinImplementor {
    output_pin: Mutex<OutputPin>,
}

impl OutputPinImplementor for PiOutputPinImplementor {
    fn write(&self, level: gpio::LogicLevel) {
        self.output_pin.lock().unwrap().write(match level {
            gpio::LogicLevel::Low => Level::Low,
            gpio::LogicLevel::High => Level::High,
        });
    }
}
