/// GPIO demo
/// Authors: Kai Page, Brendan Burmeister, Joey Vongphasouk
/// 
/// Expected Output: Have an LED blink, LED blinks faster when
/// a button input is received
/// 
/// Tested Output: LED blinks on startup and blinks faster when
/// button input received. Demo works when multiple inputs to GPIO
/// given. LED turns off at the end when demo ends

use anyhow::Result;
use gpio::*;
use std::thread;
use std::time::Duration;
wit_bindgen_rust::import!("../../wit/gpio.wit");
wit_error_rs::impl_error!(GpioError);

/// Helper function; sleeps for a set amount
/// of time depending on logic level parameter
fn sleep(input: LogicLevel) {
    thread::sleep(Duration::from_millis(match input {
        LogicLevel::Low => 500,
        LogicLevel::High => 250,
    }))
}

fn main() -> Result<()> {
    /// Define variables based on configurations in demo slightfile
    let input_pin = InputPin::get_named("push_down_button")?;
    let output_pin = OutputPin::get_named("led")?;

    /// Run infinite loop that sets logic level of output pin
    /// Call sleep function using input pin logic level
    loop {
        output_pin.write(LogicLevel::High);
        sleep(input_pin.read());
        output_pin.write(LogicLevel::Low);
        sleep(input_pin.read());
    }
}
