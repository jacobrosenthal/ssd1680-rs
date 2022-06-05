# SSD1680 driver

[![Crates.io](https://img.shields.io/crates/v/ssd1680.svg)](https://crates.io/crates/ssd1680)
[![Docs.rs](https://docs.rs/ssd1680/badge.svg)](https://docs.rs/ssd1680)

Async SPI driver for the SSD1680 Eink Mono and Tricolor displays from Adafruit

* [Adafruit 2.13" Monochrome eInk / ePaper Display FeatherWing](https://www.adafruit.com/product/4195)
* [Adafruit 2.13" HD Tri-Color eInk / ePaper Display FeatherWing - 250x122 RW Panel with SSD1680](https://www.adafruit.com/product/4814)

This driver is designed for low power usage. Sadly on both displays the Busy and Rst pin not connected and must be manually soldered.

## [Documentation](https://docs.rs/ssd1680)

## Examples

Embassy async examples are provided

`cd ssd1680-examples/feather-express-embassy/`

This crate uses [`probe-run`](https://crates.io/crates/probe-run) to run the examples. Once set up, it should be as simple as `cargo run --example <example name> --release`. `--release` will be required for some examples to reduce FLASH usage.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
