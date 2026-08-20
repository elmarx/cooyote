//! Reads the sensor stream, maintains the shared "latest measurement" state,
//! and dispatches to the HTTP module (Prometheus `/metrics`) if enabled.

use crate::config::Config;
use crate::error::CooyoteError;
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

    if config.mqtt.enabled {
        info!("TODO: connect to mqtt");
    }

    if !config.http.enabled {
        info!("HTTP server disabled via config, exiting");
        return Ok(());
    }

    crate::http::run(rx, config.http).await
}
