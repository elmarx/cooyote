# COOyote

Prometheus exporter for [TFA Dostman AIRCO2NTROL](https://www.tfa-dostmann.de/produkt/co2-monitor-airco2ntrol-mini-31-5006/).

## Cross-compiling for Raspberry Pi

I'm running this on a Raspberry Pi Zero (which is basically a Raspberry Pi 1), cross-compiling is possible via [cross](https://github.com/cross-rs/cross).

### Response example

```
# HELP co2_ppm co2 ppm
# TYPE co2_ppm gauge
co2_ppm 478
# HELP temperature_celsius temperature in celsius
# TYPE temperature_celsius gauge
temperature_celsius 22.162506103515625
```

## Implementation details

There are multiple crates implementing reading the sensor (e.g. [co2mon](https://github.com/lnicola/co2mon)).

Newer devices do not do the encryption anymore (reverse engineered [here](https://hackaday.io/project/5301-reverse-engineering-a-low-cost-usb-co-monitor)), which makes reading the device much easier (basically just decoding plain data), so it's implemented here to have full control over [hidapi](https://github.com/ruabmbua/hidapi-rs).
`hidapi` now supports [basic-udev](https://github.com/xobs/basic-udev), so compiling does not require native libraries anymore, which takes a lot of pain out of cross-compiling!
