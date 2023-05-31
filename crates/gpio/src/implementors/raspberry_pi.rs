use std::collections::hash_map::{Entry, HashMap};
use std::sync::Mutex;

use rppal::gpio::{Gpio, IoPin, Mode, Pin, PullUpDown};

use super::PinImplementor;
use crate::gpio;

pub struct RaspberryPiImplementor {
    gpio: Result<Gpio, rppal::gpio::Error>,
    inner: Mutex<RaspberryPiImplementorInner>,
}

struct RaspberryPiImplementorInner {
    pins: HashMap<u8, IoPin>,
}

/// Map a WIT mode to an rppal mode
fn map_mode(mode: gpio::Mode) -> (Mode, PullUpDown) {
    match mode {
        gpio::Mode::Output => (Mode::Output, PullUpDown::Off),
        gpio::Mode::Input => (Mode::Input, PullUpDown::Off),
        gpio::Mode::InputPullup => (Mode::Input, PullUpDown::PullUp),
        gpio::Mode::InputPulldown => (Mode::Input, PullUpDown::PullDown),
    }
}

impl RaspberryPiImplementor {
    pub fn new() -> Self {
        Self {
            gpio: Gpio::new(),
            inner: Mutex::new(RaspberryPiImplementorInner {
                pins: HashMap::new(),
            }),
        }
    }
}

impl PinImplementor for RaspberryPiImplementor {
    fn new(&self, pin: u8, mode: gpio::Mode) -> Result<u8, gpio::PinError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?;

        match inner.pins.entry(pin) {
            Entry::Occupied(_) => Err(gpio::PinError::UnexpectedError(String::from(
                "pin already allocated",
            ))),
            Entry::Vacant(e) => {
                let (mode, pud) = map_mode(mode);
                let mut io_pin = self
                    .gpio
                    .as_ref()
                    .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?
                    .get(pin)
                    .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?
                    .into_io(mode);
                io_pin.set_pullupdown(pud);
                e.insert(io_pin);
                Ok(pin)
            }
        }
    }

    fn mode(&self, pin: u8) -> crate::gpio::Mode {
        todo!()
    }

    fn set_mode(&self, pin: u8, mode: crate::gpio::Mode) {
        todo!()
    }

    fn is_output(&self, pin: u8) -> bool {
        todo!()
    }

    fn write(&self, pin: u8, level: crate::gpio::LogicLevel) {
        todo!()
    }

    fn is_input(&self, pin: u8) -> bool {
        todo!()
    }

    fn read(&self, pin: u8) -> crate::gpio::LogicLevel {
        todo!()
    }
}
