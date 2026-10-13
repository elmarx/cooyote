# COOyote

Reads CO₂ ppm and temperature from [TFA Dostman AIRCO2NTROL](https://www.tfa-dostmann.de/produkt/co2-monitor-airco2ntrol-mini-31-5006/) monitors via USB HID and publishes to Prometheus or Home Assistant via MQTT.

## Overview

Connect AIRCO2NTROL monitors and publish sensor data in two ways:

- **Prometheus:** exposes `/metrics` endpoint with CO₂ and temperature gauges
- **MQTT:** publishes readings to Home Assistant with discovery support

Both outputs can run simultaneously or independently via config.

## Configuration

Create a config file (e.g. `config.toml`) based on `config.toml.example`.

cooyote looks in the following order for config files:

- CLI argument `--config`
- Environment variable `COOYOTE_CONFIG`
- Current working directory: `config.toml`
- global `/etc/cooyote.toml`

Without a config file, cooyote defaults to Prometheus on `:8080` with MQTT disabled.

### Prometheus

```toml
[http]
enabled = true
port = 8080
```

Metrics are exposed at `http://localhost:8080/metrics`.

### MQTT

```toml
[mqtt]
enabled = true
host = "localhost"
port = 1883
topic_prefix = "cooyote"
username = "your_username"
password = "your_password"
```

Environment variable overrides: `MQTT_HOST`, `MQTT_USERNAME`, `MQTT_PASSWORD` (or `MQTT_PASSWORD_FILE` for systemd's `LoadCredential=`).

Supports [Home Assistant MQTT discovery](https://www.home-assistant.io/integrations/sensor.mqtt/).

## Installation

Build with [Rust](https://www.rust-lang.org/):

```bash
cargo install cooyote --git https://github.com/elmarx/cooyote
```

For ARM (e.g., Raspberry Pi), cross-compile using [cross](https://github.com/cross-rs/cross):

```bash
cross build --target aarch64-unknown-linux-gnu
```

## Implementation

Newer AIRCO2NTROL devices transmit unencrypted data (reverse engineered [here](https://hackaday.io/project/5301-reverse-engineering-a-low-cost-usb-co-monitor)), enabling straightforward decoding. Uses [hidapi-rs](https://github.com/ruabmbua/hidapi-rs) with [basic-udev](https://github.com/xobs/basic-udev) for USB HID access without native library dependencies.

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
