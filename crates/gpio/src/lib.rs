use std::sync::Arc;

use implementors::PinImplementor;

use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
use slight_file::Resource;

mod implementors;

wit_bindgen_wasmtime::export!("../../wit/gpio.wit");
wit_error_rs::impl_error!(gpio::PinError);
wit_error_rs::impl_from!(anyhow::Error, gpio::PinError::UnexpectedError);

#[derive(Clone)]
pub struct Gpio {
    implementor: Resource,
    pin_implementor: Arc<dyn PinImplementor>,
    capability_store: CapabilityStore<BasicState>,
}

impl Gpio {
    pub fn new(implementor: Resource, gpio: CapabilityStore<BasicState>) -> Self {
        Self {
            implementor,
            pin_implementor: match Into::<GpioImplementors>::into(implementor) {
                #[cfg(feature = "raspberry-pi")]
                GpioImplementors::RaspberryPi => todo!(),
            },
            capability_store: gpio,
        }
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
    type Pin = u8;

    fn pin_new(&mut self, number: u8, mode: gpio::Mode) -> Result<Self::Pin, gpio::PinError> {
        self.pin_implementor.new(number, mode)
    }

    fn pin_mode(&mut self, self_: &Self::Pin) -> gpio::Mode {
        self.pin_implementor.mode(*self_)
    }

    fn pin_set_mode(&mut self, self_: &Self::Pin, mode: gpio::Mode) -> () {
        self.pin_implementor.set_mode(*self_, mode)
    }

    fn pin_is_output(&mut self, self_: &Self::Pin) -> bool {
        self.pin_implementor.is_output(*self_)
    }

    fn pin_write(&mut self, self_: &Self::Pin, level: gpio::LogicLevel) -> () {
        self.pin_implementor.write(*self_, level)
    }

    fn pin_is_input(&mut self, self_: &Self::Pin) -> bool {
        self.pin_implementor.is_input(*self_)
    }

    fn pin_read(&mut self, self_: &Self::Pin) -> gpio::LogicLevel {
        self.pin_implementor.read(*self_)
    }

    fn drop_pin(&mut self, state: Self::Pin) {
        drop(state);
    }
}
