use std::collections::hash_map::{Entry, HashMap};
use std::sync::Mutex;

use rppal::gpio::{Gpio, IoPin, Level, Mode, PullUpDown};

use super::PinImplementor;
use crate::gpio;

pub struct RaspberryPiImplementor {
    gpio: Result<Gpio, rppal::gpio::Error>,
    pins: Mutex<HashMap<u8, PiPin>>,
}

struct PiPin {
    io_pin: IoPin,
    mode: gpio::Mode,
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
            pins: Mutex::new(HashMap::new()),
        }
    }
}

impl PinImplementor for RaspberryPiImplementor {
    fn new(&self, pin: u8, mode: gpio::Mode) -> Result<u8, gpio::PinError> {
        let mut pins = self
            .pins
            .lock()
            .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?;

        match pins.entry(pin) {
            Entry::Occupied(_) => Err(gpio::PinError::UnexpectedError(String::from(
                "pin already allocated",
            ))),
            Entry::Vacant(e) => {
                let (io_mode, pud) = map_mode(mode);
                let mut io_pin = self
                    .gpio
                    .as_ref()
                    .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?
                    .get(pin)
                    .map_err(|e| gpio::PinError::UnexpectedError(e.to_string()))?
                    .into_io(io_mode);
                io_pin.set_pullupdown(pud);
                e.insert(PiPin { io_pin, mode });
                Ok(pin)
            }
        }
    }

    fn mode(&self, pin: u8) -> gpio::Mode {
        let pins = self.pins.lock().unwrap();
        let pi_pin = &pins[&pin];
        pi_pin.mode
    }

    fn set_mode(&self, pin: u8, mode: gpio::Mode) {
        let mut pins = self.pins.lock().unwrap();
        let pi_pin = pins.get_mut(&pin).unwrap();

        let (io_mode, pud) = map_mode(mode);
        pi_pin.io_pin.set_mode(io_mode);
        pi_pin.io_pin.set_pullupdown(pud);
        pi_pin.mode = mode;
    }

    fn is_output(&self, pin: u8) -> bool {
        let pins = self.pins.lock().unwrap();
        let pi_pin = &pins[&pin];
        pi_pin.io_pin.mode() == Mode::Output
    }

    fn write(&self, pin: u8, level: gpio::LogicLevel) {
        let mut pins = self.pins.lock().unwrap();
        let pi_pin = pins.get_mut(&pin).unwrap();

        pi_pin.io_pin.write(match level {
            gpio::LogicLevel::High => Level::High,
            gpio::LogicLevel::Low => Level::Low,
        });
    }

    fn is_input(&self, pin: u8) -> bool {
        let pins = self.pins.lock().unwrap();
        let pi_pin = &pins[&pin];
        pi_pin.io_pin.mode() == Mode::Input
    }

    fn read(&self, pin: u8) -> gpio::LogicLevel {
        let pins = self.pins.lock().unwrap();
        let pi_pin = &pins[&pin];
        
        match pi_pin.io_pin.read() {
            Level::High => gpio::LogicLevel::High,
            Level::Low => gpio::LogicLevel::Low,
        }
    }

    fn drop(&self, pin: u8) {
        let mut pins = self.pins.lock().unwrap();
        pins.remove(&pin);
    }
}
