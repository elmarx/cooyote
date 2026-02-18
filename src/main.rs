use crate::metrics::metrics;
use axum::routing::get;
use axum::Router;
use prometheus::{Gauge, Opts, Registry};
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;

mod metrics;
mod zyaura;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sensor = zyaura::Sensor::new()?;

    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge =
        Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let r = Registry::new();
    r.register(Box::new(co2_gauge.clone()))?;
    r.register(Box::new(temperature_gauge.clone()))?;

    tokio::task::spawn_blocking(move || loop {
        let measurement = sensor.read();

        match measurement {
            Ok(measurement) => {
                temperature_gauge.set(f64::from(measurement.temperature));
                co2_gauge.set(f64::from(measurement.co2));
            }
            Err(e) => {
                eprintln!("Error reading measurement: {e}");
            }
        }

        thread::sleep(Duration::from_secs(60));
    });

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(Arc::new(r));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
