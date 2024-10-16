use axum::extract::State;
use axum::{debug_handler};
use prometheus::{Encoder, Registry, TextEncoder};
use std::str::FromStr;
use std::sync::Arc;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;

#[debug_handler]
pub async fn metrics(State(registry): State<Arc<Registry>>) -> (TypedHeader<ContentType>, Vec<u8>) {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        TypedHeader(ContentType::from_str(encoder.format_type()).unwrap()),
        buffer,
    )
}
