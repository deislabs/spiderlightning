use anyhow::Result;

use gpio::*;
wit_bindgen_rust::import!("../../wit/gpio.wit");
wit_error_rs::impl_error!(GpioError);

fn main() -> Result<()> {
    let input_pin = InputPin::get_named("push_down_button")?;
    let output_pin = OutputPin::get_named("led")?;

    loop {
        output_pin.write(input_pin.read());
    }

    // println!("Hello, world!");
}
