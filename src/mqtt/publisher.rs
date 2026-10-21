use crate::config::{default_discovery_expire, MqttConfig};
use crate::error::CooyoteError;
use crate::mqtt::event_handler::MqttEventHandler;
use crate::zyaura::Measurement;
use rumqttc::v5::mqttbytes::v5::PublishProperties;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::AsyncClient;
use serde_json::json;
use tokio::sync::watch;
use tracing::{error, info};

/// Publishes sensor readings, Home Assistant auto-discovery messages and
/// availability status to MQTT. Also acts as the `MqttEventHandler`, reacting to
/// reconnects (republish discovery + availability + last known reading).
#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
    topic_prefix: String,
    ha_topic: String,
    availability_topic: String,
    rx: watch::Receiver<Measurement>,
    publish_properties: PublishProperties,
}

impl MqttEventHandler for MqttPublisher {
    async fn on_mqtt_connection_online(&self) {
        if let Err(e) = self.publish_discovery().await {
            error!(error = %e, "publishing discovery message failed");
        }

        if let Err(e) = self.publish_online().await {
            error!(error = %e, "publishing \"online\" availability failed");
        }

        // republish the last known reading, so retained state topics aren't stale
        // after having been disconnected for a while
        let measurement = *self.rx.borrow();
        if let Err(e) = self.publish_measurements(measurement).await {
            error!(error = %e, "republishing last known measurement failed");
        }
    }
}

impl MqttPublisher {
    pub fn new(config: &MqttConfig, client: AsyncClient, rx: watch::Receiver<Measurement>) -> Self {
        Self {
            client,
            topic_prefix: config.topic_prefix.clone(),
            availability_topic: config.availability_topic.clone(),
            ha_topic: config.homeassistant_topic.clone(),
            rx,
            // publish properties, as we retain messages, but do not want to keep them forever
            publish_properties: PublishProperties {
                message_expiry_interval: Some(
                    u32::try_from(config.discovery_expire.as_secs()).unwrap_or_else(|_| {
                        u32::try_from(default_discovery_expire().as_secs()).unwrap_or(u32::MAX)
                    }),
                ),
                ..Default::default()
            },
        }
    }

    pub async fn publish_discovery(&self) -> Result<(), CooyoteError> {
        self.publish_sensor_discovery("co2", "CO2", "carbon_dioxide", "ppm")
            .await?;
        self.publish_sensor_discovery("temperature", "Temperature", "temperature", "°C")
            .await?;
        info!("published discovery messages");
        Ok(())
    }

    async fn publish_sensor_discovery(
        &self,
        name: &str,
        friendly_name: &str,
        device_class: &str,
        unit: &str,
    ) -> Result<(), CooyoteError> {
        let discovery_payload = json!({
            "name": friendly_name,
            "unique_id": format!("cooyote_{name}"),
            "state_topic": format!("{}/{name}/state", self.topic_prefix),
            "unit_of_measurement": unit,
            "device_class": device_class,
            "state_class": "measurement",
            "availability_topic": self.availability_topic,
            "payload_available": "online",
            "payload_not_available": "offline",
            "device": {
                "identifiers": ["cooyote"],
                "name": "Cooyote CO2 Monitor",
                "model": "AIRCO2NTROL Mini",
                "manufacturer": "TFA Dostmann",
            }
        });

        let discovery_topic = format!("{}/sensor/cooyote/{name}/config", self.ha_topic);

        self.client
            .publish_with_properties(
                discovery_topic,
                QoS::AtLeastOnce,
                true,
                discovery_payload.to_string(),
                self.publish_properties.clone(),
            )
            .await
            .map_err(|e| CooyoteError::MqttHaDiscovery(Box::new(e)))
    }

    /// Publishes both readings (co2 + temperature) contained in `measurement`.
    pub async fn publish_measurements(&self, measurement: Measurement) -> Result<(), CooyoteError> {
        self.publish_measurement("co2", f64::from(measurement.co2))
            .await?;
        self.publish_measurement("temperature", f64::from(measurement.temperature))
            .await?;
        Ok(())
    }

    async fn publish_measurement(&self, sensor_name: &str, value: f64) -> Result<(), CooyoteError> {
        let state_topic = format!("{}/{sensor_name}/state", self.topic_prefix);

        self.client
            .publish_with_properties(
                state_topic,
                QoS::AtLeastOnce,
                true,
                value.to_string(),
                self.publish_properties.clone(),
            )
            .await
            .map_err(|e| CooyoteError::MqttSensorPublish(Box::new(e)))
    }

    async fn publish_online(&self) -> Result<(), CooyoteError> {
        self.client
            .publish_with_properties(
                &self.availability_topic,
                QoS::AtLeastOnce,
                true,
                "online",
                self.publish_properties.clone(),
            )
            .await
            .map_err(|e| CooyoteError::MqttOnlinePublish(Box::new(e)))
    }
}
