# GPIO support

This crate provides the ability for SpiderLighting applications to interact with GPIO pins.

Pins are named in the slightfile and passed a configuration string with the following slash-separated parameters:

- Pin number
- Pin mode
  - `input`
  - `output`
  - `pwm_output`
- Optional mode-specific configuration
  - For input: `pullup` or `pulldown`
  - For output: initially set to `low` or `high`
  - For PWM output: the PWM period in microseconds, if supported by the implementation

See the [example application](../../examples/gpio-demo/).
