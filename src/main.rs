use tracing::{info, Level};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
    
    pub fn built_time() -> built::chrono::DateTime<built::chrono::Local> {
        built::util::strptime(BUILT_TIME_UTC)
            .with_timezone(&built::chrono::offset::Local)
    }
}

mod conf;
mod state;
mod gpio;
mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let config = conf::Config::new()?;
    let url = config.url();
    
    let app = api::create_app(config)?;

    let listener = tokio::net::TcpListener::bind(&url).await?;
    info!("Listening on {url}...");
    axum::serve(listener, app).await?;

    Ok(())
}
