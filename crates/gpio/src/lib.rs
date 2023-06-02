use std::collections::hash_map::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use implementors::*;

use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
use slight_file::resource::GpioResource::*;
use slight_file::Resource;

mod implementors;

wit_bindgen_wasmtime::export!("../../wit/gpio.wit");
wit_error_rs::impl_error!(gpio::GpioError);

// needs to be Send + Sync
#[derive(Clone)]
pub struct Gpio {
    pins: HashMap<String, Result<Pin, gpio::GpioError>>,
}

#[derive(Clone)]
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
                    .map(|(name, config)| {
                        (
                            name.clone(),
                            (|| {
                                let mut config_iter = config.split('/');

                                let pin_number = config_iter.next().ok_or_else(|| {
                                    gpio::GpioError::ConfigurationError(String::from(
                                        "no pin number",
                                    ))
                                })?;
                                let pin_number = u8::from_str(pin_number).map_err(|e| {
                                    gpio::GpioError::ConfigurationError(format!(
                                        "invalid pin number: {e}"
                                    ))
                                })?;

                                match config_iter.next() {
                                    Some("input") => {
                                        Ok(Pin::Input(implementor.new_input_pin(pin_number)?))
                                    }
                                    Some("output") => {
                                        Ok(Pin::Output(implementor.new_output_pin(pin_number)?))
                                    }
                                    Some(unknown_type) => Err(gpio::GpioError::ConfigurationError(
                                        format!("unknown pin type '{unknown_type}'"),
                                    )),
                                    None => Err(gpio::GpioError::ConfigurationError(String::from(
                                        "no pin type",
                                    ))),
                                }
                            })(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        Self { pins }
    }
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

impl gpio::Gpio for Gpio {
    type InputPin = Arc<dyn InputPinImplementor>;
    type OutputPin = Arc<dyn OutputPinImplementor>;

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

    fn input_pin_read(&mut self, self_: &Self::InputPin) -> gpio::LogicLevel {
        self_.read()
    }

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

    fn output_pin_write(&mut self, self_: &Self::OutputPin, level: gpio::LogicLevel) -> () {
        self_.write(level)
    }
}
