use axum::debug_handler;
use axum::extract::State;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;
use prometheus::{Encoder, Registry, TextEncoder};
use std::str::FromStr;
use std::sync::Arc;

#[debug_handler]
pub async fn metrics(State(registry): State<Arc<Registry>>) -> (TypedHeader<ContentType>, Vec<u8>) {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    // there is a limited set of metrics, so unwrapping here should be safe
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("failed to encode metrics");

    // the format_type() points to a const value, so
    let content_type = ContentType::from_str(encoder.format_type()).expect("invalid content type");

    (TypedHeader(content_type), buffer)
}
