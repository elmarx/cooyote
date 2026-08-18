use axum::debug_handler;
use axum::extract::State;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::watch;

use crate::zyaura::Measurement;

pub struct AppState {
    pub registry: Registry,
    pub rx: watch::Receiver<Measurement>,
    pub co2_gauge: Gauge,
    pub temperature_gauge: Gauge,
}

#[debug_handler]
pub async fn metrics(State(state): State<Arc<AppState>>) -> (TypedHeader<ContentType>, Vec<u8>) {
    // query the shared state for the latest measurement and reflect it in the gauges
    let measurement = *state.rx.borrow();
    state.co2_gauge.set(f64::from(measurement.co2));
    state
        .temperature_gauge
        .set(f64::from(measurement.temperature));

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = state.registry.gather();
    // there is a limited set of metrics, so unwrapping here should be safe
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("failed to encode metrics");

    // the format_type() points to a const value, so
    let content_type = ContentType::from_str(encoder.format_type()).expect("invalid content type");

    (TypedHeader(content_type), buffer)
}
