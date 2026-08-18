//! Reads readings from the sensor in a loop and prints them to stdout

use crate::zyaura;
use tracing::{error, info};

pub fn run(sensor: &zyaura::Sensor) {
    loop {
        match sensor.read() {
            Ok(measurement) => info!(?measurement, "read measurement"),
            Err(e) => error!(error = %e, "error reading measurement"),
        }
    }
}
