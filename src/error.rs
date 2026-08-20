use crate::{config, zyaura};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CooyoteError {
    #[error(transparent)]
    Sensor(#[from] zyaura::Error),

    #[error(transparent)]
    Config(#[from] config::ConfigError),

    #[error("sensor stream ended unexpectedly")]
    SensorStreamEnded,

    #[error("failed to set up Prometheus metrics: {0}")]
    Metrics(#[from] prometheus::Error),

    #[error("failed to bind HTTP server to {address}: {source}")]
    HttpBind {
        address: String,
        #[source]
        source: std::io::Error,
    },

    #[error("HTTP server error: {0}")]
    HttpServe(#[source] std::io::Error),
}
