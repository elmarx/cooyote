use crate::metrics::{metrics, AppState};
use crate::zyaura::{Error as SensorError, Measurement};
use axum::routing::get;
use axum::Router;
use prometheus::{Gauge, Opts, Registry};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_stream::{Stream, StreamExt};
use tracing::{error, info};

pub async fn run(
    mut stream: impl Stream<Item = Result<Measurement, SensorError>> + Unpin + Send + 'static,
) -> Result<(), Box<dyn Error>> {
    // seed the watch channel with the first successful reading, so we don't serve "0"
    // and to make sure we can read the sensor at all before starting the HTTP server
    let initial_measurement = loop {
        match stream.next().await {
            Some(Ok(measurement)) => break measurement,
            Some(Err(e)) => error!(error = %e, "error reading measurement"),
            None => return Err("sensor stream ended unexpectedly".into()),
        }
    };

    let (tx, rx) = watch::channel(initial_measurement);

    // forward further readings from the stream into the watch channel, so the HTTP
    // handler can just query the latest value on each request instead of awaiting the stream itself
    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(measurement) => {
                    // ignore send errors: no receivers left just means the server has shut down
                    let _ = tx.send(measurement);
                }
                Err(e) => error!(error = %e, "error reading measurement"),
            }
        }
    });

    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge =
        Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let registry = Registry::new();
    registry.register(Box::new(co2_gauge.clone()))?;
    registry.register(Box::new(temperature_gauge.clone()))?;

    let state = Arc::new(AppState {
        registry,
        rx,
        co2_gauge,
        temperature_gauge,
    });

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!(address = "0.0.0.0:8080", "listening");
    axum::serve(listener, app).await?;

    Ok(())
}
