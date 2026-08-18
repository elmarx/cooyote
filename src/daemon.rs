use crate::metrics::metrics;
use crate::zyaura;
use axum::routing::get;
use axum::Router;
use prometheus::{Gauge, Opts, Registry};
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn run(sensor: zyaura::Sensor) -> Result<(), Box<dyn Error>> {
    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge =
        Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let r = Registry::new();
    r.register(Box::new(co2_gauge.clone()))?;
    r.register(Box::new(temperature_gauge.clone()))?;

    // set an initial measurement to not serve "0" and make sure we can read the sensor
    let initial_measurement = sensor.read()?;
    temperature_gauge.set(f64::from(initial_measurement.temperature));
    co2_gauge.set(f64::from(initial_measurement.co2));

    tokio::task::spawn_blocking(move || loop {
        let measurement = sensor.read();

        match measurement {
            Ok(measurement) => {
                temperature_gauge.set(f64::from(measurement.temperature));
                co2_gauge.set(f64::from(measurement.co2));
            }
            Err(e) => {
                error!(error = %e, "error reading measurement");
            }
        }

        thread::sleep(Duration::from_mins(1));
    });

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(Arc::new(r));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!(address = "0.0.0.0:8080", "listening");
    axum::serve(listener, app).await?;

    Ok(())
}
