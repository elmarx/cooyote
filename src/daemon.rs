//! Reads the sensor stream, maintains the shared "latest measurement" state,
//! and dispatches to the HTTP module (Prometheus `/metrics`) if enabled.

use crate::config::Config;
use crate::error::CooyoteError;
use crate::mqtt;
use crate::zyaura::{Error as SensorError, Measurement};
use tokio::sync::watch;
use tokio_stream::{Stream, StreamExt};
use tracing::{error, info};

pub async fn run(
    mut stream: impl Stream<Item = Result<Measurement, SensorError>> + Unpin + Send + 'static,
    config: Config,
) -> Result<(), CooyoteError> {
    // seed the watch channel with the first successful reading, so we don't serve "0"
    // and to make sure we can read the sensor at all before starting the HTTP server
    let initial_measurement = loop {
        match stream.next().await {
            Some(Ok(measurement)) => break measurement,
            Some(Err(e)) => error!(error = %e, "error reading measurement"),
            None => return Err(CooyoteError::SensorStreamEnded),
        }
    };

    let (tx, rx) = watch::channel(initial_measurement);

    // forward further readings from the stream into the watch channel, so consumers
    // (e.g. the HTTP handler) can just query the latest value instead of awaiting the stream themselves
    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(measurement) => {
                    // ignore send errors: no receivers left just means the server has shut down
                    let _ = tx.send(measurement);
                }
                Err(e) => error!(error = %e, "error reading measurement"),
            }
        }
    });

    // if MQTT is enabled, spawn its connection + publish loop in the background;
    // we need the handle so we can keep the process alive on it if HTTP is disabled
    let mqtt_handle = if config.mqtt.enabled {
        let mqtt_rx = rx.clone();
        let mqtt_config = config.mqtt; // partial move: config.http below stays valid
        Some(tokio::spawn(async move {
            if let Err(e) = mqtt::run(mqtt_config, mqtt_rx).await {
                error!(error = %e, "MQTT publishing stopped");
            }
        }))
    } else {
        None
    };

    if !config.http.enabled {
        info!("HTTP server disabled via config");
        return if let Some(handle) = mqtt_handle {
            info!("running with MQTT only");
            handle.await.map_err(CooyoteError::MqttTaskFailed)
        } else {
            info!("MQTT disabled too, exiting");
            Ok(())
        };
    }

    crate::http::run(rx, config.http).await
}
