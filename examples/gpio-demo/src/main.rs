use anyhow::Result;

use gpio::*;
use std::thread;
use std::time::Duration;
wit_bindgen_rust::import!("../../wit/gpio.wit");
wit_error_rs::impl_error!(GpioError);

fn sleep(input: LogicLevel) {
    thread::sleep(Duration::from_millis(match input {
        LogicLevel::Low => 500,
        LogicLevel::High => 250,
    }))
}

fn main() -> Result<()> {
    let input_pin = InputPin::get_named("push_down_button")?;
    let output_pin = OutputPin::get_named("led")?;

    loop {
        output_pin.write(LogicLevel::High);
        sleep(input_pin.read());
        output_pin.write(LogicLevel::Low);
        sleep(input_pin.read());
    }
}
