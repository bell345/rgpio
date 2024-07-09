use std::time::Duration;
use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tower_http::trace::{TraceLayer};
use tracing::{info};
use crate::gpio::IGpio;
use crate::state::AppState;

pub fn create_app(config: crate::conf::Config) -> anyhow::Result<Router> {
    let state = AppState::new(config)?;

    Ok(Router::new()
        .nest("/api", Router::new()
            .route("/", get(root))
            .route("/version", get(version))
            .route("/gpio/:pin", get(read_pin))
            .route("/gpio/:pin/on", put(turn_pin_on))
            .route("/gpio/:pin/off", put(turn_pin_off))
            .route("/gpio/:pin/press", put(press_pin))
            .layer(TraceLayer::new_for_http())
            .with_state(state))
    )
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

enum ApiError {
    BadRequest(String),
    GpioReadError(u8, <crate::gpio::Gpio as IGpio>::Error),
    GpioWriteError(u8, <crate::gpio::Gpio as IGpio>::Error)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(s) =>
                (StatusCode::BAD_REQUEST, s),
            ApiError::GpioReadError(pin, gpio_error) =>
                (StatusCode::INTERNAL_SERVER_ERROR,
                 format!("Error while reading pin {pin}: {gpio_error}")),
            ApiError::GpioWriteError(pin, gpio_error) =>
                (StatusCode::INTERNAL_SERVER_ERROR,
                 format!("Error while writing pin {pin}: {gpio_error}"))
        }.into_response()
    }
}

fn check_pin_range(pin: u8) -> Result<(), ApiError> {
    if pin > 53 {
        return Err(ApiError::BadRequest(format!("Pin number {pin} is out of range")));
    }

    Ok(())
}

async fn read_pin(
    State(state): State<AppState>,
    Path(pin): Path<u8>
) -> Result<Json<PinState>, ApiError> {
    check_pin_range(pin)?;

    let mut gpio = state.gpio.lock().await;
    gpio.get(pin)
        .map(|l| Json(PinState { pin, value: l.into() }))
        .map_err(|e| ApiError::GpioReadError(pin, e))
}

async fn turn_pin_on(
    State(state): State<AppState>,
    Path(pin): Path<u8>
) -> Result<(), ApiError> {
    check_pin_range(pin)?;

    let mut gpio = state.gpio.lock().await;
    gpio.set(pin, true.into())
        .map_err(|e| ApiError::GpioWriteError(pin, e))
}

async fn turn_pin_off(
    State(state): State<AppState>,
    Path(pin): Path<u8>
) -> Result<(), ApiError> {
    check_pin_range(pin)?;

    let mut gpio = state.gpio.lock().await;
    gpio.set(pin, false.into())
        .map_err(|e| ApiError::GpioWriteError(pin, e))
}

#[derive(Debug, Clone, Deserialize)]
struct PressOptions {
    delay_ms: Option<u64>
}

async fn press_pin(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
    Query(options): Query<PressOptions>
) -> Result<(), ApiError> {
    check_pin_range(pin)?;

    let delay_ms = options.delay_ms.unwrap_or(200);
    if delay_ms > state.config.max_press_delay_ms {
        return Err(ApiError::BadRequest(
                format!("Provided delay of {} is greater than max delay {}",
                        delay_ms, state.config.max_press_delay_ms)));
    }

    let mut gpio = state.gpio.lock().await;
    gpio.set(pin, true.into())
        .map_err(|e| ApiError::GpioWriteError(pin, e))?;

    drop(gpio);
    sleep(Duration::from_millis(delay_ms)).await;

    gpio = state.gpio.lock().await;
    gpio.set(pin, false.into())
        .map_err(|e| ApiError::GpioWriteError(pin, e))?;

    Ok(())
}
