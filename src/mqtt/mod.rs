mod event_handler;
mod eventloop;
mod init_connection;
mod publisher;

use crate::config::MqttConfig;
use crate::error::CooyoteError;
use crate::zyaura::Measurement;
use publisher::MqttPublisher;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;
use tracing::info;

/// Connects to the configured MQTT broker and continuously publishes new sensor
/// readings (plus Home Assistant discovery/availability) for as long as `rx`
/// keeps producing values.
pub async fn run(config: MqttConfig, rx: watch::Receiver<Measurement>) -> Result<(), CooyoteError> {
    info!(host = %config.host, port = config.port, "connecting to MQTT broker");

    let (client, eventloop) = init_connection::init_client(&config).await?;
    let publisher = MqttPublisher::new(&config, client, rx.clone());

    eventloop::spawn(eventloop, publisher.clone());

    // publish every new reading (step-1 cadence: no change-detection/min-interval yet, see plan.md)
    let mut stream = WatchStream::new(rx);
    while let Some(measurement) = stream.next().await {
        if let Err(e) = publisher.publish_measurements(measurement).await {
            tracing::error!(error = %e, "failed to publish measurement to MQTT");
        }
    }

    Ok(())
}
