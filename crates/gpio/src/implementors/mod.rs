use crate::gpio::*;

#[cfg(feature = "raspberry-pi")]
pub mod raspberry_pi;

pub trait PinImplementor: Send + Sync {
    fn new(&self, pin: u8, mode: Mode) -> Result<u8, PinError>;

    fn mode(&self, pin: u8) -> Mode;
    fn set_mode(&self, pin: u8, mode: Mode);

    fn is_output(&self, pin: u8) -> bool;
    fn write(&self, pin: u8, level: LogicLevel);

    fn is_input(&self, pin: u8) -> bool;
    fn read(&self, pin: u8) -> LogicLevel;

    fn drop(&self, pin: u8);
}

impl std::fmt::Debug for dyn PinImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinImplementor").finish_non_exhaustive()
    }
}
