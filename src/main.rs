extern crate core;

use paho_mqtt as mqtt;
use std::time::Duration;
use std::{env, thread};

fn main() {
    let mqtt_host = env::var("MQTT_HOST").unwrap_or("localhost".to_string());
    let mqtt_user = env::var("MQTT_USER").ok();
    let mqtt_password = env::var("MQTT_PASSWORD").ok();
    let cli =
        mqtt::Client::new(format!("tcp://{mqtt_host}:1883")).expect("error creating the client");

    let mut connect_options = mqtt::ConnectOptionsBuilder::new();
    if let Some(mqtt_user) = mqtt_user {
        connect_options.user_name(mqtt_user);
    }
    if let Some(mqtt_password) = mqtt_password {
        connect_options.password(mqtt_password);
    }
    connect_options.will_message(mqtt::Message::new("home/co2-mqtt/state", "offline", 0));

    let _ = cli
        .connect(connect_options.finalize())
        .expect("Unable to connect");
    cli.publish(mqtt::Message::new_retained(
        "home/co2-mqtt/state",
        "online1",
        1,
    ))
    .expect("Error sending message");

    let sensor = co2mon::Sensor::open_default().expect("error opening sensor");

    loop {
        match sensor.read() {
            Ok(reading) => {
                cli.publish(mqtt::Message::new(
                    "home/co2-mqtt/temperature",
                    reading.temperature().to_string(),
                    0,
                ))
                .expect("error sending temperature value");
                cli.publish(mqtt::Message::new(
                    "home/co2-mqtt/co2",
                    reading.co2().to_string(),
                    0,
                ))
                .expect("error sending co2 value");
            }
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(60));
    }
}
