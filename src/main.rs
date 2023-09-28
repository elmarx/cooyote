use std::thread;
use std::time::Duration;

fn main() {
    let sensor = co2mon::Sensor::open_default().expect("error opening sensor");

    loop {
        match sensor.read() {
            Ok(reading) => println!("{}°C {}ppm", reading.temperature(), reading.co2()),
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(60));
    }
}
