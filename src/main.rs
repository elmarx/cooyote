use crate::cli::{Args, Commands};
use crate::error::CooyoteError;
use clap::Parser;
use tracing::info;

mod cli;
mod config;
mod daemon;
mod debug;
mod error;
mod http;
mod zyaura;

#[tokio::main]
async fn main() -> Result<(), CooyoteError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Config always has sensible defaults (HTTP enabled, MQTT disabled), even if no
    // config file is found.
    let config = config::get_config_path(args.config)
        .map(config::Config::from_file)
        .transpose()?
        .unwrap_or_default();

    info!(?config, "Loaded configuration");

    let sensor = zyaura::Sensor::new()?;
    info!("Connected to sensor");

    let stream = sensor.stream();

    match args.command {
        Some(Commands::Debug) => {
            debug::run(stream).await;
            Ok(())
        }
        None => daemon::run(stream, config).await,
    }
}
