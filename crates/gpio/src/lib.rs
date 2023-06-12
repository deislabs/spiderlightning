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

/// Implements the GPIO interface defined by gpio.wit.
///
/// This structure is responsible for constructing the pin resources described in the slightfile and providing them to the application upon request.
///
/// It must be [Send], [Sync], and [Clone].
#[derive(Clone)]
pub struct Gpio {
    pins: HashMap<String, Result<Pin, gpio::GpioError>>,
}

/// A type for storing constructed pin resources.
///
/// There should be one variant for each pin type, holding an [Arc] reference to the implementor trait object.
#[derive(Debug, Clone)]
enum Pin {
    Input(Arc<dyn InputPinImplementor>),
    Output(Arc<dyn OutputPinImplementor>),
    PwmOutput(Arc<dyn PwmOutputPinImplementor>),
}

impl Gpio {
    /// Construct a new [Gpio] object.
    ///
    /// This function reads in the pin descriptors from the named state in `capability_store`.
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

/// Directions that internal resistors can be configured to pull a floating wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pull {
    Up,
    Down,
}

/// A list of GPIO implementations that the slightfile can refer to.
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
    type PwmOutputPin = Arc<dyn PwmOutputPinImplementor>;

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

    fn pwm_output_pin_get_named(
        &mut self,
        name: &str,
    ) -> Result<Self::PwmOutputPin, gpio::GpioError> {
        match self.pins.get(name) {
            Some(Ok(Pin::PwmOutput(pin))) => Ok(pin.clone()),
            Some(Ok(_)) => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not a PWM output pin"
            ))),
            Some(Err(e)) => Err(e.clone()),
            None => Err(gpio::GpioError::PinUsageError(format!(
                "'{name}' is not a named pin"
            ))),
        }
    }

    fn pwm_output_pin_set_duty_cycle(&mut self, self_: &Self::PwmOutputPin, duty_cycle: f32) -> () {
        self_.set_duty_cycle(if duty_cycle.is_nan() {
            0.0
        } else {
            duty_cycle.clamp(0.0, 1.0)
        })
    }

    fn pwm_output_pin_enable(&mut self, self_: &Self::PwmOutputPin) -> () {
        self_.enable()
    }

    fn pwm_output_pin_disable(&mut self, self_: &Self::PwmOutputPin) -> () {
        self_.disable()
    }
}
