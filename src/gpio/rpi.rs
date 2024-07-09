use crate::gpio::IGpio;

#[derive(Debug, Clone)]
pub struct Gpio(Box<rppal::gpio::Gpio>);

impl From<rppal::gpio::Level> for crate::gpio::mock::Level {
    fn from(value: rppal::gpio::Level) -> Self {
        match value {
            rppal::gpio::Level::High => crate::gpio::mock::Level::High,
            rppal::gpio::Level::Low => crate::gpio::mock::Level::Low
        }
    }
}

impl IGpio for Gpio {
    type Error = rppal::gpio::Error;
    type Level = crate::gpio::mock::Level;
    fn new() -> Result<Self, Self::Error> {
        Ok(Self(rppal::gpio::Gpio::new()?.into()))
    }

    fn get(&mut self, pin: u8) -> Result<Self::Level, Self::Error> {
        Ok(self.0.get(pin)?.read().into())
    }

    fn set(&mut self, pin: u8, level: Self::Level) -> Result<(), Self::Error> {
        let mut out = self.0.get(pin)?.into_output();
        out.set_reset_on_drop(false);
        match level {
            Self::Level::High => out.set_high(),
            Self::Level::Low => out.set_low()
        }
        Ok(())
    }
}
