use axum::routing::get;
use axum::{Router, Server};
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::metrics::metrics;
use prometheus::{Gauge, Opts, Registry};

mod metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sensor = co2mon::Sensor::open_default()?;

    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge =
        Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let r = Registry::new();
    r.register(Box::new(co2_gauge.clone())).unwrap();
    r.register(Box::new(temperature_gauge.clone())).unwrap();

    tokio::task::spawn_blocking(move || loop {
        match sensor.read() {
            Ok(reading) => {
                co2_gauge.set(reading.co2() as f64);
                temperature_gauge.set(reading.temperature() as f64);
            }
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(60));
    });

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(Arc::new(r));

    Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
