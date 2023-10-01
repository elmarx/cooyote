use std::error::Error;
use std::thread;
use std::time::Duration;

use prometheus::{Encoder, Gauge, Opts, Registry, TextEncoder};

fn main() -> Result<(), Box<dyn Error>> {
    let sensor = co2mon::Sensor::open_default()?;

    let co2_gauge = Gauge::with_opts(Opts::new("co2_ppm", "co2 ppm"))?;
    let temperature_gauge = Gauge::with_opts(Opts::new("temperature_celsius", "temperature in celsius"))?;

    let r = Registry::new();
    r.register(Box::new(co2_gauge.clone())).unwrap();
    r.register(Box::new(temperature_gauge.clone())).unwrap();

    loop {
        match sensor.read() {
            Ok(reading) => {
                co2_gauge.set(reading.co2() as f64);
                temperature_gauge.set(reading.temperature() as f64);

                let mut buffer = vec![];
                let encoder = TextEncoder::new();
                let metric_families = r.gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();

                println!("{}", String::from_utf8(buffer).unwrap());
            },
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(60));
    }
}
