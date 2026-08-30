use crate::config::MqttConfig;
use rumqttc::v5::mqttbytes::v5::LastWill;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, EventLoop, MqttOptions};
use std::time::Duration;

/// Sets up the async MQTT client + event loop: configures a last-will-and-testament
/// (marking the device "offline" on unclean disconnects).
pub async fn init_client(config: &MqttConfig) -> (AsyncClient, EventLoop) {
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
        None,
    );
    mqtt_options.set_last_will(last_will);

    AsyncClient::new(mqtt_options, 10)
}
