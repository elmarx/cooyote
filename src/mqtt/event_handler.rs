use bytes::Bytes;
use std::future::Future;

/// Reacts to incoming MQTT events. Implemented directly by `MqttPublisher`,
/// since publishing discovery/availability messages is a reaction to these
/// events. Not `dyn`-dispatched, so async fns in this trait are fine.
pub trait MqttEventHandler: Send + Sync {
    /// called when a subscribed topic publishes a new message (here: `{homeassistant_topic}/status`)
    fn on_mqtt_message(&self, topic: String, payload: Bytes) -> impl Future<Output = ()> + Send;

    /// called when the mqtt connection becomes available: initial connect and every reconnect
    fn on_mqtt_connection_online(&self) -> impl Future<Output = ()> + Send;
}
