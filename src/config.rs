//! Configuration file: currently used to configure the HTTP (Prometheus
//! `/metrics`) server and to enable publishing readings to MQTT / Home
//! Assistant. `Config` always has sensible defaults (HTTP enabled, MQTT
//! disabled), so it can be relied upon even if no config file is found.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[source] std::io::Error),
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[source] toml::de::Error),
}

/// A wrapper type for sensitive values that hides the content when formatted with Debug
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Secret<T>(pub T);

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("***")
    }
}

impl<T> std::ops::Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub mqtt: MqttConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HttpConfig {
    /// whether to serve the Prometheus `/metrics` endpoint at all
    pub enabled: bool,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8080,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MqttConfig {
    /// whether to publish readings to MQTT / Home Assistant at all
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<Secret<String>>,
    /// prefix for the state topics, e.g. `{topic_prefix}/co2/state`
    pub topic_prefix: String,
    /// availability (online/offline) topic, set via last-will-and-testament
    pub availability_topic: String,
    /// homeassistant topic, used for discovery and status, defaults to homeassistant
    pub homeassistant_topic: String,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_mqtt_port(),
            username: None,
            password: None,
            topic_prefix: default_topic_prefix(),
            availability_topic: default_availability_topic(),
            homeassistant_topic: default_homeassistant_topic(),
        }
    }
}

impl MqttConfig {
    /// Override username and password from environment variables if set
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(username) = std::env::var("MQTT_USERNAME") {
            self.username = Some(username);
        }
        if let Ok(password) = std::env::var("MQTT_PASSWORD") {
            self.password = Some(Secret(password));
        } else if let Ok(password_file) = std::env::var("MQTT_PASSWORD_FILE") {
            // supports e.g. systemd's LoadCredential=/Environment=..%d/.. mechanism,
            // where the secret is provided as a file instead of directly as an
            // environment variable
            if let Ok(password) = fs::read_to_string(password_file) {
                self.password = Some(Secret(password.trim_end().to_string()));
            }
        }

        if let Ok(host) = std::env::var("MQTT_HOST") {
            self.host = host;
        }

        self
    }
}

fn default_mqtt_port() -> u16 {
    1883
}

fn default_topic_prefix() -> String {
    "cooyote".to_string()
}

fn default_availability_topic() -> String {
    "cooyote/availability".to_string()
}

fn default_homeassistant_topic() -> String {
    "homeassistant".to_string()
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path).map_err(ConfigError::ReadError)?;
        let mut config: Config = toml::from_str(&contents).map_err(ConfigError::ParseError)?;

        config.mqtt = config.mqtt.with_env_overrides();

        Ok(config)
    }
}

/// Resolve the config file path: explicit CLI arg > `COOYOTE_CONFIG` env var >
/// `./config.toml` > `/etc/cooyote.toml`. Returns `None` if none of these
/// exist, in which case `Config::default()` is used.
pub fn get_config_path(config_from_cli: Option<PathBuf>) -> Option<PathBuf> {
    config_from_cli
        .or_else(|| std::env::var("COOYOTE_CONFIG").map(PathBuf::from).ok())
        .or_else(|| {
            let config_in_cwd = PathBuf::from("config.toml");
            config_in_cwd.exists().then_some(config_in_cwd)
        })
        .or_else(|| {
            let config_in_etc = PathBuf::from("/etc/cooyote.toml");
            config_in_etc.exists().then_some(config_in_etc)
        })
}
