use std::sync::Arc;
use std::time;
use anyhow::Context;
use crate::gpio::IGpio;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<crate::conf::Config>,
    pub gpio: Arc<Mutex<crate::gpio::Gpio>>,
    pub request_id_cache: moka::sync::Cache<Uuid, ()>
}

impl AppState {
    pub fn new(config: crate::conf::Config) -> anyhow::Result<Self> {
        Ok(Self {
            config: Arc::new(config),
            gpio: Arc::new(Mutex::new(
                crate::gpio::Gpio::new()
                    .context("Failed to initialise GPIO")?
            )),
            request_id_cache: moka::sync::CacheBuilder::default()
                .time_to_live(time::Duration::from_secs(60 * 5))
                .build()
        })
    }
}