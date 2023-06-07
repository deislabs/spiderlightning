use std::collections::hash_map::HashMap;
use std::sync::Arc;

use implementors::*;

use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
use slight_file::resource::GpioResource::*;
use slight_file::Resource;

mod implementors;
#[cfg(test)]
mod tests;

wit_bindgen_wasmtime::export!("../../wit/gpio.wit");
wit_error_rs::impl_error!(gpio::GpioError);

// needs to be Send + Sync
#[derive(Clone)]
pub struct Gpio {
    pins: HashMap<String, Result<Pin, gpio::GpioError>>,
}

#[derive(Debug, Clone)]
enum Pin {
    Input(Arc<dyn InputPinImplementor>),
    Output(Arc<dyn OutputPinImplementor>),
}

impl Gpio {
    pub fn new(name: &str, capability_store: CapabilityStore<BasicState>) -> Self {
        let state = capability_store.get(name, "gpio").unwrap().clone();
        let mut implementor: Box<dyn GpioImplementor> =
            match GpioImplementors::from(state.implementor) {
                #[cfg(feature = "raspberry_pi")]
                GpioImplementors::RaspberryPi => Box::new(raspberry_pi::PiGpioImplementor::new()),
            };

        let pins = state
            .configs_map
            .map(|configs| {
                configs
                    .iter()
                    .map(|(name, config)| (name.clone(), implementor.parse_pin_config(config)))
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        Self { pins }
    }
}
//enum for pullUp/pullDown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pull {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum GpioImplementors {
    #[cfg(feature = "raspberry_pi")]
    RaspberryPi,
}

impl From<Resource> for GpioImplementors {
    fn from(s: Resource) -> Self {
        match s {
            #[cfg(feature = "raspberry_pi")]
            Resource::Gpio(RaspberryPi) => Self::RaspberryPi,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
            ),
        }
    }
}

impl_resource!(
    Gpio,
    gpio::GpioTables<Gpio>,
    gpio::add_to_linker,
    "gpio".to_string()
);

///converts between the wit and slight file config to be used
impl gpio::Gpio for Gpio {
    type InputPin = Arc<dyn InputPinImplementor>;
    type OutputPin = Arc<dyn OutputPinImplementor>;

    ///for input pins, gives the pin number
    fn input_pin_get_named(&mut self, name: &str) -> Result<Self::InputPin, gpio::GpioError> {
        match self.pins.get(name) {
            Some(Ok(Pin::Input(pin))) => Ok(pin.clone()),
            Some(Ok(_)) => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not an input pin"
            ))),
            Some(Err(e)) => Err(e.clone()),
            None => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not a named pin"
            ))),
        }
    }
    ///read the LogicLevel from pin (high/low)
    fn input_pin_read(&mut self, self_: &Self::InputPin) -> gpio::LogicLevel {
        self_.read()
    }
    
    ///for output pins, gives the pin number
    fn output_pin_get_named(&mut self, name: &str) -> Result<Self::OutputPin, gpio::GpioError> {
        match self.pins.get(name) {
            Some(Ok(Pin::Output(pin))) => Ok(pin.clone()),
            Some(Ok(_)) => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not an output pin"
            ))),
            Some(Err(e)) => Err(e.clone()),
            None => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not a named pin"
            ))),
        }
    }
    ///for output pins, stores the logic level
    fn output_pin_write(&mut self, self_: &Self::OutputPin, level: gpio::LogicLevel) -> () {
        self_.write(level)
    }
}
