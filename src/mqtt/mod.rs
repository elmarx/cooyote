mod event_handler;
mod eventloop;
mod init_connection;
mod publisher;

use crate::config::MqttConfig;
use crate::error::CooyoteError;
use crate::zyaura::Measurement;
use publisher::MqttPublisher;
use std::time::Duration;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;
use tracing::{error, info};

/// Connects to the configured MQTT broker and continuously publishes new sensor
/// readings (plus Home Assistant discovery/availability) for as long as `rx`
/// keeps producing values.
pub async fn run(config: MqttConfig, rx: watch::Receiver<Measurement>) -> Result<(), CooyoteError> {
    info!(host = %config.host, port = config.port, "connecting to MQTT broker");

    // we publish discovery messages with expiry, so we have to republish periodically
    let discovery_republish_interval = config
        .discovery_expire
        .saturating_sub(Duration::from_hours(1));

    let (client, eventloop) = init_connection::init_client(&config).await;
    let publisher = MqttPublisher::new(&config, client, rx.clone());

    eventloop::spawn(eventloop, publisher.clone());

    let mut discovery_republish = tokio::time::interval(discovery_republish_interval);
    // the first tick fires immediately; we already publish discovery on connect, so skip it
    discovery_republish.tick().await;

    // publish every new reading (step-1 cadence: no change-detection/min-interval yet, see plan.md)
    let mut stream = WatchStream::new(rx);
    loop {
        tokio::select! {
            measurement = stream.next() => {
                match measurement {
                    Some(measurement) => {
                        if let Err(e) = publisher.publish_measurements(measurement).await {
                            error!(error = %e, "failed to publish measurement to MQTT");
                        }
                    }
                    None => break,
                }
            }
            _ = discovery_republish.tick() => {
                if let Err(e) = publisher.publish_discovery().await {
                    error!(error = %e, "periodic re-publish of discovery message failed");
                } else {
                    info!("periodically re-published discovery message");
                }
            }
        }
    }

    Ok(())
}
