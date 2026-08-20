use crate::mqtt::event_handler::MqttEventHandler;
use rumqttc::{Event, EventLoop, Incoming};
use std::time::Duration;
use tracing::error;

const INITIAL_BACKOFF: Duration = Duration::from_millis(100);
const MAX_BACKOFF: Duration = Duration::from_mins(1);
const BACKOFF_MULTIPLIER: u32 = 2;

/// Spawns a background task continuously polling the MQTT event loop, dispatching
/// incoming messages/connection events to `handler`, and reconnecting with an
/// exponential backoff on error (rumqttc handles the actual reconnect, we just
/// need to keep calling `poll()`).
pub fn spawn(mut eventloop: EventLoop, handler: impl MqttEventHandler + 'static) {
    tokio::spawn(async move {
        handle_connection(&mut eventloop, &handler).await;
    });
}

async fn handle_connection<H: MqttEventHandler>(eventloop: &mut EventLoop, handler: &H) {
    let mut current_backoff = INITIAL_BACKOFF;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                current_backoff = INITIAL_BACKOFF;
                handler.on_mqtt_connection_online().await;
            }
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                handler
                    .on_mqtt_message(publish.topic, publish.payload)
                    .await;
            }
            Ok(_) => {}
            Err(e) => {
                error!(error = %e, backoff_ms = current_backoff.as_millis(), "MQTT connection error, backing off");
                tokio::time::sleep(current_backoff).await;
                current_backoff = current_backoff
                    .saturating_mul(BACKOFF_MULTIPLIER)
                    .min(MAX_BACKOFF);
            }
        }
    }
}
