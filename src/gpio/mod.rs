pub trait IGpio: Sized + Clone {
    type Error: std::error::Error;
    type Level: Into<bool> + From<bool>;

    fn new() -> Result<Self, Self::Error>;
    fn get(&mut self, pin: u8) -> Result<Self::Level, Self::Error>;
    fn set(&mut self, pin: u8, level: Self::Level) -> Result<(), Self::Error>;
}

#[cfg(any(target_arch = "arm", target_arch = "armv7", target_arch = "aarch64"))]
mod rpi;

#[cfg(any(target_arch = "arm", target_arch = "armv7", target_arch = "aarch64"))]
pub type Gpio = rpi::Gpio;

mod mock;

#[cfg(target_arch = "x86_64")]
pub type Gpio = mock::Gpio;
