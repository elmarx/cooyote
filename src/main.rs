use crate::metrics::metrics;
use crate::zyaura::Item;
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
        let temp = sensor.read_item();

        match temp {
            Ok(Item::Temperature(t)) => temperature_gauge.set(f64::from(t)),
            Ok(Item::CO2(c)) => temperature_gauge.set(f64::from(c)),
            _ => {}
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
