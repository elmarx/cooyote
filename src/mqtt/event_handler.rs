use std::future::Future;

/// Reacts to incoming MQTT events. Implemented directly by `MqttPublisher`,
/// since publishing discovery/availability messages is a reaction to these
/// events. Not `dyn`-dispatched, so async fns in this trait are fine.
pub trait MqttEventHandler: Send + Sync {
    /// called when the mqtt connection becomes available: initial connect and every reconnect
    fn on_mqtt_connection_online(&self) -> impl Future<Output = ()> + Send;
}
