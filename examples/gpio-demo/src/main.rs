//! GPIO demo
//! Authors: Kai Page, Brendan Burmeister, Joey Vongphasouk
//!
//! Expected result: One LED blinks, and blinks faster while its
//! corresponding input is high. The other LED gets brighter while
//! its corresponding input is high and darker while its
//! corresponding input is low.

use anyhow::Result;
use gpio::*;
use std::thread;
use std::time::Duration;
wit_bindgen_rust::import!("../../wit/gpio.wit");
wit_error_rs::impl_error!(GpioError);

const BLINK_THRESHOLD: u32 = 500;

fn main() -> Result<()> {
    // Define variables based on configurations in demo slightfile
    let input_pin = InputPin::get_named("push_down_button")?;
    let output_pin = OutputPin::get_named("led")?;
    let pwm_control_pin = InputPin::get_named("pwm_control_button")?;
    let pwm_output_pin = PwmOutputPin::get_named("pwm_led")?;

    let mut blink_progress = 0;
    let mut blink_current = LogicLevel::High;
    let mut pwm_duty_cycle: f32 = 0.0;

    output_pin.write(LogicLevel::High);
    pwm_output_pin.set_duty_cycle(0.0);
    pwm_output_pin.enable();

    // Run infinite loop that updates outputs based on inputs
    loop {
        blink_progress += match input_pin.read() {
            LogicLevel::Low => 1,
            LogicLevel::High => 2,
        };
        if blink_progress >= BLINK_THRESHOLD {
            blink_current = match blink_current {
                LogicLevel::Low => LogicLevel::High,
                LogicLevel::High => LogicLevel::Low,
            };
            output_pin.write(blink_current);
            blink_progress = 0;
        }

        pwm_duty_cycle = (pwm_duty_cycle
            + match pwm_control_pin.read() {
                LogicLevel::Low => -0.001,
                LogicLevel::High => 0.001,
            })
        .clamp(0.0, 1.0);
        pwm_output_pin.set_duty_cycle(pwm_duty_cycle);

        thread::sleep(Duration::from_millis(1));
    }
}
