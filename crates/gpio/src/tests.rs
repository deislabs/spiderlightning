use std::sync::Arc;

use crate::implementors::*;
use crate::{gpio, Pull};

#[derive(Default)]
struct MockGpioImplementor {
    last_construction: Option<MockPin>,
}

impl GpioImplementor for MockGpioImplementor {
    fn new_input_pin(
        &mut self,
        pin: u8,
        pull: Option<Pull>,
    ) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError> {
        let pin = MockPin::Input { pin, pull };
        self.last_construction.replace(pin);
        Ok(Arc::new(pin))
    }

    fn new_output_pin(
        &mut self,
        pin: u8,
        init_level: Option<gpio::LogicLevel>,
    ) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError> {
        let pin = MockPin::Output { pin, init_level };
        self.last_construction.replace(pin);
        Ok(Arc::new(pin))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockPin {
    Input {
        pin: u8,
        pull: Option<Pull>,
    },
    Output {
        pin: u8,
        init_level: Option<gpio::LogicLevel>,
    },
}

impl InputPinImplementor for MockPin {
    fn read(&self) -> gpio::LogicLevel {
        gpio::LogicLevel::Low
    }
}

impl OutputPinImplementor for MockPin {
    fn write(&self, _: gpio::LogicLevel) {}
}

#[test]
fn good_pin_configs() {
    let mut gpio = MockGpioImplementor::default();
    for (config, expected) in [
        ("1/input", MockPin::Input { pin: 1, pull: None }),
        (
            "2/input/pullup",
            MockPin::Input {
                pin: 2,
                pull: Some(Pull::Up),
            },
        ),
        (
            "3/input/pulldown",
            MockPin::Input {
                pin: 3,
                pull: Some(Pull::Down),
            },
        ),
        (
            "4/output",
            MockPin::Output {
                pin: 4,
                init_level: None,
            },
        ),
        (
            "5/output/low",
            MockPin::Output {
                pin: 5,
                init_level: Some(gpio::LogicLevel::Low),
            },
        ),
        (
            "6/output/high",
            MockPin::Output {
                pin: 6,
                init_level: Some(gpio::LogicLevel::High),
            },
        ),
    ] {
        let result = (&mut gpio as &mut dyn GpioImplementor).parse_pin_config(config);
        assert!(result.is_ok(), "good config '{config}' returned {result:?}");
        match gpio.last_construction {
            Some(actual) => assert_eq!(
                expected, actual,
                "config '{config}': expected {expected:?}, got {actual:?}"
            ),
            None => panic!("no pin constructed for '{config}' (result is {result:?})"),
        }
    }
}

#[test]
fn bad_pin_configs() {
    let mut gpio = MockGpioImplementor::default();
    let gpio: &mut dyn GpioImplementor = &mut gpio;
    for config in [
        "",
        "some",
        "body/once",
        "told/me/the",
        "1/world",
        "1/input/was",
        "1/output/gonna",
        "1/input/pullup/roll",
        "1/output/low/me",
        "-1/input",
        "420/output",
    ] {
        match gpio.parse_pin_config(config) {
            Err(gpio::GpioError::ConfigurationError(_)) => (),
            Err(wrong) => {
                panic!(
                    "bad config '{config}' returned {wrong:?}, which is not a ConfigurationError"
                )
            }
            Ok(pin) => panic!("bad config '{config}' somehow returned Ok({pin:?}"),
        }
    }
}
