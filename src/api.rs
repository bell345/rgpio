use crate::gpio::IGpio;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use axum_extra::headers::Header;
use axum_extra::{headers, TypedHeader};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

pub fn create_app(config: crate::conf::Config) -> anyhow::Result<Router> {
    let state = AppState::new(config)?;

    Ok(Router::new().nest(
        "/api",
        Router::new()
            .route("/", get(root))
            .route("/version", get(version))
            .route("/gpio/:pin", get(read_pin))
            .route("/gpio/:pin/on", put(turn_pin_on))
            .route("/gpio/:pin/off", put(turn_pin_off))
            .route("/gpio/:pin/press", put(press_pin))
            .route("/gpio/:pin/press-twice", put(press_pin_twice))
            .layer(TraceLayer::new_for_http())
            .with_state(state),
    ))
}

async fn root(State(state): State<AppState>) -> String {
    let motd = state.config.motd.clone();
    info!("Serving MOTD: {motd}");
    motd
}

#[derive(Debug, Clone, Serialize)]
struct VersionInfo {
    version: String,
    build_time: String,
    git_commit: Option<String>,
}

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        version: crate::built_info::PKG_VERSION.into(),
        build_time: crate::built_info::built_time().to_rfc3339(),
        git_commit: crate::built_info::GIT_COMMIT_HASH_SHORT.map(|s| s.into()),
    })
}

#[derive(Debug, Clone, Serialize)]
struct PinState {
    pin: u8,
    value: bool,
}

#[derive(Debug, Clone)]
struct RequestIdHeader(Uuid);

static REQUEST_ID_HEADER_NAME: HeaderName = HeaderName::from_static("x-request-id");

impl Header for RequestIdHeader {
    fn name() -> &'static HeaderName {
        &REQUEST_ID_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.last().ok_or(headers::Error::invalid())?;
        let uuid = Uuid::try_parse_ascii(value.as_ref()).map_err(|_| headers::Error::invalid())?;
        Ok(RequestIdHeader(uuid))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend([self.0.to_string().try_into().unwrap()])
    }
}

enum ApiError {
    BadRequest(String),
    DuplicateRequest(Uuid),
    GpioReadError(u8, <crate::gpio::Gpio as IGpio>::Error),
    GpioWriteError(u8, <crate::gpio::Gpio as IGpio>::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(s) => (StatusCode::BAD_REQUEST, s),
            ApiError::DuplicateRequest(id) => (
                StatusCode::CONFLICT,
                format!("A request with the same ID ({id}) has already been made"),
            ),
            ApiError::GpioReadError(pin, gpio_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while reading pin {pin}: {gpio_error}"),
            ),
            ApiError::GpioWriteError(pin, gpio_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while writing pin {pin}: {gpio_error}"),
            ),
        }
        .into_response()
    }
}

fn check_pin_range(pin: u8) -> Result<(), ApiError> {
    if pin > 53 {
        return Err(ApiError::BadRequest(format!(
            "Pin number {pin} is out of range"
        )));
    }

    Ok(())
}

fn check_request_id(
    app_state: &AppState,
    request_id_header: &Option<TypedHeader<RequestIdHeader>>,
) -> Result<(), ApiError> {
    let Some(TypedHeader(RequestIdHeader(request_id))) = request_id_header.as_ref() else {
        return Ok(());
    };

    if app_state.request_id_cache.contains_key(request_id) {
        return Err(ApiError::DuplicateRequest(*request_id));
    }

    app_state.request_id_cache.insert(*request_id, ());

    Ok(())
}

async fn write_gpio(state: &AppState, pin: u8, value: bool) -> Result<(), ApiError> {
    let mut gpio = state.gpio.lock().await;
    gpio.set(pin, value)
        .map(|_| Ok(()))
        .map_err(|e| ApiError::GpioWriteError(pin, e))?
}

async fn read_pin(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
) -> Result<Json<PinState>, ApiError> {
    check_pin_range(pin)?;

    let mut gpio = state.gpio.lock().await;
    gpio.get(pin)
        .map(|l| {
            Json(PinState {
                pin,
                value: l.into(),
            })
        })
        .map_err(|e| ApiError::GpioReadError(pin, e))
}

async fn turn_pin_on(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
    request_id_opt: Option<TypedHeader<RequestIdHeader>>,
) -> Result<(), ApiError> {
    check_pin_range(pin)?;
    check_request_id(&state, &request_id_opt)?;

    write_gpio(&state, pin, true).await
}

async fn turn_pin_off(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
    request_id_opt: Option<TypedHeader<RequestIdHeader>>,
) -> Result<(), ApiError> {
    check_pin_range(pin)?;
    check_request_id(&state, &request_id_opt)?;

    write_gpio(&state, pin, false).await
}

#[derive(Debug, Clone, Deserialize)]
struct PressOptions {
    delay_ms: Option<u64>,
}

async fn do_press(state: &AppState, pin: u8, press_delay: Duration) -> Result<(), ApiError> {
    write_gpio(state, pin, true).await?;

    sleep(press_delay).await;

    write_gpio(state, pin, false).await
}

fn parse_duration_ms(
    kind: &'static str,
    value_opt: Option<u64>,
    default: u64,
    maximum: u64,
) -> Result<Duration, ApiError> {
    let value_ms = value_opt.unwrap_or(default);

    if value_ms > maximum {
        return Err(ApiError::BadRequest(format!(
            "Provided {kind} of {value_ms} ms is greater than max delay {maximum} ms",
        )));
    }

    Ok(Duration::from_millis(value_ms))
}

async fn press_pin(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
    request_id_opt: Option<TypedHeader<RequestIdHeader>>,
    Query(options): Query<PressOptions>,
) -> Result<(), ApiError> {
    check_pin_range(pin)?;
    check_request_id(&state, &request_id_opt)?;

    let delay = parse_duration_ms(
        "delay",
        options.delay_ms,
        state.config.default_press_delay_ms,
        state.config.max_press_delay_ms,
    )?;

    do_press(&state, pin, delay).await
}

#[derive(Debug, Clone, Deserialize)]
struct PressTwiceOptions {
    press_delay_ms: Option<u64>,
    press_interval_ms: Option<u64>,
}

async fn press_pin_twice(
    State(state): State<AppState>,
    Path(pin): Path<u8>,
    request_id_opt: Option<TypedHeader<RequestIdHeader>>,
    Query(options): Query<PressTwiceOptions>,
) -> Result<(), ApiError> {
    check_pin_range(pin)?;
    check_request_id(&state, &request_id_opt)?;

    let press_delay = parse_duration_ms(
        "press delay",
        options.press_delay_ms,
        state.config.default_press_delay_ms,
        state.config.max_press_delay_ms,
    )?;

    let press_interval = parse_duration_ms(
        "press interval",
        options.press_interval_ms,
        state.config.default_press_interval_ms,
        state.config.max_press_interval_ms,
    )?;

    do_press(&state, pin, press_delay).await?;

    sleep(press_interval).await;

    do_press(&state, pin, press_delay).await
}
