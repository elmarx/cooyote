# COOyote

Prometheus exporter for [TFA Dostman AIRCO2NTROL](https://www.tfa-dostmann.de/produkt/co2-monitor-airco2ntrol-mini-31-5006/).

Basically, this is [co2mon](https://github.com/lnicola/co2mon) plugged into [axum](https://github.com/tokio-rs/axum#axum) (and of course prometheus), nothing spectacular.

This repo serves mostly as an example, but feel free to open issues for feature requests if that makes it reusable for you.

## Cross-compiling for Raspberry Pi

I'm running this on a Raspberry Pi Zero (which is basically a Raspberry Pi 1), cross-compiling is possible via [cross](https://github.com/cross-rs/cross).

## Response example

```
# HELP co2_ppm co2 ppm
# TYPE co2_ppm gauge
co2_ppm 478
# HELP temperature_celsius temperature in celsius
# TYPE temperature_celsius gauge
temperature_celsius 22.162506103515625
```