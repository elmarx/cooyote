//! Consumes the sensor's reading stream directly and prints every reading
//! (or error) to stdout via tracing. Does not start the HTTP server or
//! publish anything to MQTT.

use tokio_stream::{Stream, StreamExt};
use tracing::{error, info};

use crate::zyaura::{Error, Measurement};

pub async fn run(mut stream: impl Stream<Item = Result<Measurement, Error>> + Unpin) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(measurement) => info!(?measurement, "read measurement"),
            Err(e) => error!(error = %e, "error reading measurement"),
        }
    }
}
