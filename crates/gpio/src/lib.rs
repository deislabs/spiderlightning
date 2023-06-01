use std::collections::hash_map::HashMap;
use std::sync::Arc;

use implementors::{InputPinImplementor, OutputPinImplementor};

use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
use slight_file::Resource;

mod implementors;

wit_bindgen_wasmtime::export!("../../wit/gpio.wit");
wit_error_rs::impl_error!(gpio::GpioError);

// needs to be Send + Sync
#[derive(Clone)]
pub struct Gpio {
    capability_store: CapabilityStore<BasicState>,
    pins: HashMap<String, Result<Pin, gpio::GpioError>>,
}

#[derive(Clone)]
enum Pin {
    Input(Arc<dyn InputPinImplementor>),
    Output(Arc<dyn OutputPinImplementor>),
}

impl Gpio {
    pub fn new(implementor: Resource, gpio: CapabilityStore<BasicState>) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum GpioImplementors {
    #[cfg(feature = "raspberry-pi")]
    RaspberryPi,
}

impl From<Resource> for GpioImplementors {
    fn from(s: Resource) -> Self {
        todo!()
    }
}

impl_resource!(
    Gpio,
    gpio::GpioTables<Gpio>,
    gpio::add_to_linker,
    "gpio".to_string()
);

impl gpio::Gpio for Gpio {
    type InputPin = Arc<dyn InputPinImplementor>;
    type OutputPin = Arc<dyn OutputPinImplementor>;

    fn input_pin_get_named(&mut self, name: &str) -> Result<Self::InputPin, gpio::GpioError> {
        match self.pins.get(name) {
            Some(Ok(Pin::Input(pin))) => Ok(pin.clone()),
            Some(Ok(_)) => Err(gpio::GpioError::PinUsageError(format!("'{name}' is not an input pin"))),
            Some(Err(e)) => Err(e.clone()),
            None => Err(gpio::GpioError::PinUsageError(format!("'{name}' is not a named pin"))),
        }
    }

    fn input_pin_read(&mut self, self_: &Self::InputPin) -> gpio::LogicLevel {
        self_.read()
    }

    fn output_pin_get_named(&mut self, name: &str) -> Result<Self::OutputPin, gpio::GpioError> {
        match self.pins.get(name) {
            Some(Ok(Pin::Output(pin))) => Ok(pin.clone()),
            Some(Ok(_)) => Err(gpio::GpioError::PinUsageError(format!("'{name}' is not an output pin"))),
            Some(Err(e)) => Err(e.clone()),
            None => Err(gpio::GpioError::PinUsageError(format!("'{name}' is not a named pin"))),
        }
    }

    fn output_pin_write(&mut self, self_: &Self::OutputPin, level: gpio::LogicLevel) -> () {
        self_.write(level)
    }
}
