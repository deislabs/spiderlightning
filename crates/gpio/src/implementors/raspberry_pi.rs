// use std::collections::hash_map::{Entry, HashMap};
use std::sync::Mutex;

use rppal::gpio::{Gpio, InputPin, Level, OutputPin};

use super::*;
use crate::gpio;

pub struct PiGpioImplementor {
    gpio: Result<Gpio, gpio::GpioError>,
}

impl PiGpioImplementor {
    fn new() -> Self {
        Self {
            gpio: Gpio::new().map_err(|e| gpio::GpioError::HardwareError(e.to_string())),
        }
    }
}

impl GpioImplementor for PiGpioImplementor {
    fn new_input_pin(&mut self, pin: u8) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let input_pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?
            .into_input();
        Ok(Arc::new(PiInputPinImplementor { input_pin }))
    }

    fn new_output_pin(
        &mut self,
        pin: u8,
    ) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let output_pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?
            .into_output();
        let output_pin = Mutex::new(output_pin);
        Ok(Arc::new(PiOutputPinImplementor { output_pin }))
    }
}

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
