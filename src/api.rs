use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::get;
use serde::Serialize;
use tower_http::trace::{TraceLayer};
use tracing::{info};
use crate::gpio::IGpio;
use crate::state::AppState;

pub fn create_app(config: crate::conf::Config) -> anyhow::Result<Router> {
    let state = AppState::new(config)?;

    Ok(Router::new()
        .route("/", get(root))
        .route("/version", get(version))
        .route("/gpio/:pin", get(read_pin))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

async fn root(
    State(state): State<AppState>
) -> String {
    let motd = state.config.motd.clone();
    info!("Serving MOTD: {motd}");
    motd
}

#[derive(Debug, Clone, Serialize)]
struct VersionInfo {
    version: String,
    build_time: String,
    git_commit: Option<String>
}

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        version: crate::built_info::PKG_VERSION.into(),
        build_time: crate::built_info::built_time().to_rfc3339(),
        git_commit: crate::built_info::GIT_COMMIT_HASH_SHORT.map(|s| s.into())
    })
}

#[derive(Debug, Clone, Serialize)]
struct PinState {
    pin: u8,
    value: bool
}

async fn read_pin(
    State(state): State<AppState>,
    Path(pin): Path<u8>
) -> Result<Json<PinState>, (StatusCode, String)> {
    if pin > 53 {
        return Err((StatusCode::BAD_REQUEST, format!("Pin number {pin} is out of range")));
    }
    
    state.gpio.lock().get(pin)
        .map(|l| Json(PinState { pin, value: l.into() }))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error while reading pin {pin}: {e}")))
}
