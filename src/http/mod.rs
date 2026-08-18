//! The Prometheus `/metrics` HTTP endpoint.

mod metrics;

use crate::config::HttpConfig;
use crate::zyaura::Measurement;
use axum::routing::get;
use axum::Router;
use metrics::AppState;
use prometheus::{Gauge, Opts, Registry};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing::info;

/// Starts serving the `/metrics` endpoint, reflecting the latest measurement
/// from `rx` on every request.
pub async fn run(
    rx: watch::Receiver<Measurement>,
    http_config: HttpConfig,
) -> Result<(), Box<dyn Error>> {
    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge =
        Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let registry = Registry::new();
    registry.register(Box::new(co2_gauge.clone()))?;
    registry.register(Box::new(temperature_gauge.clone()))?;

    let state = Arc::new(AppState {
        registry,
        rx,
        co2_gauge,
        temperature_gauge,
    });

    let app = Router::new()
        .route("/metrics", get(metrics::metrics))
        .with_state(state);

    let address = format!("0.0.0.0:{}", http_config.port);
    let listener = TcpListener::bind(&address).await?;
    info!(%address, "listening");
    axum::serve(listener, app).await?;

    Ok(())
}
