use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::gpio::IGpio;

#[derive(Clone, Debug, Default)]
pub struct Gpio {
    pin_state: HashMap<u8, Level>
}

#[derive(Debug)]
pub struct Error { }

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Level {
    Low = 0,
    High = 1
}

impl From<Level> for bool {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => false,
            Level::High => true
        }
    }
}

impl From<bool> for Level {
    fn from(value: bool) -> Self {
        match value {
            true => Level::High,
            false => Level::Low
        }
    }
}

impl IGpio for Gpio {
    type Error = Error;
    type Level = Level;

    fn new() -> Result<Self, Error> {
        Ok(Default::default())
    }

    fn get(&mut self, pin: u8) -> Result<Level, Error> {
        Ok(*self.pin_state.entry(pin).or_insert(Level::Low))
    }

    fn set(&mut self, pin: u8, level: Level) -> Result<(), Error> {
        self.pin_state.insert(pin, level);
        Ok(())
    }
}
