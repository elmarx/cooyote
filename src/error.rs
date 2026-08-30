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

    #[error("Failed to publish MQTT HomeAssistant sensor autodiscovery message: {0}")]
    MqttHaDiscovery(#[source] Box<rumqttc::v5::ClientError>),

    #[error("Failed to publish MQTT sensor measurement: {0}")]
    MqttSensorPublish(#[source] Box<rumqttc::v5::ClientError>),

    #[error("Failed to publish MQTT availability message: {0}")]
    MqttOnlinePublish(#[source] Box<rumqttc::v5::ClientError>),

    #[error("MQTT task panicked: {0}")]
    MqttTaskFailed(#[from] tokio::task::JoinError),

    #[error("failed to bind HTTP server to {address}: {source}")]
    HttpBind {
        address: String,
        #[source]
        source: std::io::Error,
    },

    #[error("HTTP server error: {0}")]
    HttpServe(#[source] std::io::Error),
}
