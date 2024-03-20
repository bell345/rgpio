use std::sync::Arc;
use anyhow::Context;
use crate::gpio::IGpio;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: crate::conf::Config,
    pub gpio: Arc<Mutex<crate::gpio::Gpio>>
}

impl AppState {
    pub fn new(config: crate::conf::Config) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            gpio: Arc::new(Mutex::new(
                crate::gpio::Gpio::new()
                    .context("Failed to initialise GPIO")?
            ))
        })
    }
}