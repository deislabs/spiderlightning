use std::ops::DerefMut;
use std::sync::Mutex;
use std::time::Duration;

use rppal::gpio::{Gpio, InputPin, Level, OutputPin};

use super::*;
use crate::{gpio, Pull};

/// A GPIO implementation backed by the [rppal] crate.
pub(crate) struct PiGpioImplementor {
    gpio: Result<Gpio, gpio::GpioError>,
}

impl PiGpioImplementor {
    /// Construct a new [PiGpioImplementor].
    ///
    /// If constructing the underlying [Gpio] object fails, the error will be stored for reporting to the application code.
    pub(crate) fn new() -> Self {
        Self {
            gpio: Gpio::new().map_err(|e| gpio::GpioError::HardwareError(e.to_string())),
        }
    }
}

impl GpioImplementor for PiGpioImplementor {
    fn new_input_pin(
        &mut self,
        pin: u8,
        pull: Option<Pull>,
    ) -> Result<Arc<dyn InputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?;
        let input_pin = match pull {
            Some(Pull::Up) => pin.into_input_pullup(),
            Some(Pull::Down) => pin.into_input_pulldown(),
            None => pin.into_input(),
        };
        Ok(Arc::new(PiInputPinImplementor { input_pin }))
    }

    fn new_output_pin(
        &mut self,
        pin: u8,
        init_level: Option<gpio::LogicLevel>,
    ) -> Result<Arc<dyn OutputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let mut output_pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?
            .into_output();
        match init_level {
            Some(gpio::LogicLevel::Low) => output_pin.set_low(),
            Some(gpio::LogicLevel::High) => output_pin.set_high(),
            None => (),
        }
        let output_pin = Mutex::new(output_pin);
        Ok(Arc::new(PiOutputPinImplementor { output_pin }))
    }

    fn new_pwm_output_pin(
        &mut self,
        pin: u8,
        period_microseconds: Option<u32>,
    ) -> Result<Arc<dyn PwmOutputPinImplementor>, gpio::GpioError> {
        let gpio = self.gpio.as_mut().map_err(|e| e.clone())?;
        let output_pin = gpio
            .get(pin)
            .map_err(|e| gpio::GpioError::HardwareError(e.to_string()))?
            .into_output();

        let period = Duration::from_micros(period_microseconds.unwrap_or(1000) as u64);
        let pulse_width = period / 2;

        Ok(Arc::new(PiPwmOutputPinImplementor {
            inner: Mutex::new(PiPwmInner {
                output_pin,
                period,
                pulse_width,
                enabled: false,
            }),
        }))
    }
}

/// An input pin resource backed by [rppal]'s [InputPin].
pub struct PiInputPinImplementor {
    input_pin: InputPin,
}

impl InputPinImplementor for PiInputPinImplementor {
    fn read(&self) -> gpio::LogicLevel {
        match self.input_pin.read() {
            Level::Low => gpio::LogicLevel::Low,
            Level::High => gpio::LogicLevel::High,
        }
    }
}

/// An output pin resource backed by [rppal]'s [OutputPin].
pub struct PiOutputPinImplementor {
    output_pin: Mutex<OutputPin>,
}

impl OutputPinImplementor for PiOutputPinImplementor {
    fn write(&self, level: gpio::LogicLevel) {
        self.output_pin.lock().unwrap().write(match level {
            gpio::LogicLevel::Low => Level::Low,
            gpio::LogicLevel::High => Level::High,
        });
    }
}

/// A PWM output pin resource backed by [rppal]'s [OutputPin]'s software PWM.
pub struct PiPwmOutputPinImplementor {
    inner: Mutex<PiPwmInner>,
}

struct PiPwmInner {
    output_pin: OutputPin,
    period: Duration,
    pulse_width: Duration,
    enabled: bool,
}

impl PwmOutputPinImplementor for PiPwmOutputPinImplementor {
    fn set_duty_cycle(&self, duty_cycle: f32) {
        let mut inner = self.inner.lock().unwrap();
        let PiPwmInner {
            output_pin,
            period,
            pulse_width,
            enabled,
        } = inner.deref_mut();
        // panic safety: duty_cycle is defo a finite number between 0.0 and 1.0, so this can't go negative or overflow
        *pulse_width = period.mul_f32(duty_cycle);
        if *enabled {
            if let Err(e) = output_pin.set_pwm(*period, *pulse_width) {
                tracing::error!("error enabling Raspberry Pi PWM: {e}");
            }
        }
    }

    fn enable(&self) {
        let mut inner = self.inner.lock().unwrap();
        let PiPwmInner {
            output_pin,
            period,
            pulse_width,
            enabled,
        } = inner.deref_mut();
        *enabled = true;
        if let Err(e) = output_pin.set_pwm(*period, *pulse_width) {
            tracing::error!("error enabling Raspberry Pi PWM: {e}");
        }
    }

    fn disable(&self) {
        let mut inner = self.inner.lock().unwrap();
        let PiPwmInner {
            output_pin,
            enabled,
            ..
        } = inner.deref_mut();
        *enabled = false;
        if let Err(e) = output_pin.clear_pwm() {
            tracing::error!("error disabling Raspberry Pi PWM: {e}");
        }
    }
}
