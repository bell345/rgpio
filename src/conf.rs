use std::env;
use anyhow::Context;
use serde::Deserialize;
use tracing::info;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub motd: String,
    pub max_press_delay_ms: u64
}

impl Config {
    pub(crate) fn new() -> anyhow::Result<Config> {
        dotenvy::dotenv().ok();

        let args: Vec<String> = env::args().collect();
        let conf_filename = args.get(1).cloned()
            .or_else(|| env::var("RGPIO_CONFIG_FILE").ok())
            .unwrap_or("rgpio.toml".into());

        info!("Using config filename {conf_filename}...");
        let config: Config = config::Config::builder()
            .add_source(config::File::with_name(&conf_filename)
                .format(config::FileFormat::Toml))
            .add_source(config::Environment::with_prefix("RGPIO")
                .convert_case(config::Case::Snake))
            .build().context("Failed to resolve config sources")?
            .try_deserialize().context("Failed to build config")?;
        
        Ok(config)
    }
    
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}