use crate::cli::{Args, Commands};
use clap::Parser;
use std::error::Error;
use tracing::info;

mod cli;
mod daemon;
mod debug;
mod metrics;
mod zyaura;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let sensor = zyaura::Sensor::new()?;
    info!("Connected to sensor");

    let stream = sensor.stream();

    match args.command {
        Some(Commands::Debug) => {
            debug::run(stream).await;
            Ok(())
        }
        None => daemon::run(stream).await,
    }
}
