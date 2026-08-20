use crate::config::MqttConfig;
use rumqttc::{AsyncClient, EventLoop, LastWill, MqttOptions, QoS};
use std::time::Duration;

/// Sets up the async MQTT client + event loop: configures a last-will-and-testament
/// (marking the device "offline" on unclean disconnects) and subscribes to the
/// Home Assistant status topic, so we know when to (re-)publish discovery messages.
pub async fn init_client(
    config: &MqttConfig,
) -> Result<(AsyncClient, EventLoop), rumqttc::ClientError> {
    let mut mqtt_options = MqttOptions::new("cooyote", &config.host, config.port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        mqtt_options.set_credentials(username.as_str(), password.as_str());
    }

    let last_will = LastWill::new(
        &config.availability_topic,
        "offline",
        QoS::AtLeastOnce,
        true,
    );
    mqtt_options.set_last_will(last_will);

    let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

    let ha_status_topic = format!("{}/status", config.homeassistant_topic);
    client.subscribe(ha_status_topic, QoS::AtMostOnce).await?;

    Ok((client, eventloop))
}
