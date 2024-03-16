#![allow(dead_code, unused)]

use std::error::Error;
use std::thread;
use std::time::Duration;
#[cfg(any(target_arch = "arm", target_arch = "armv7", target_arch = "aarch64"))]
use {
    rppal::gpio::Gpio,
    rppal::system::DeviceInfo
};

const GPIO_LED: u8 = 23;

#[cfg(any(target_arch = "arm", target_arch = "armv7", target_arch = "aarch64"))]
fn main() -> Result<(), Box<dyn Error>> {
    /*println!("Blinking an LED on a {}.", DeviceInfo::new()?.model());

    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    pin.set_high();
    thread::sleep(Duration::from_millis(500));
    pin.set_low();*/

    println!("Hello from ARM!");
    thread::sleep(Duration::from_secs(999_999_999));

    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn main() {
    println!("Hello from x86_64!");
}
