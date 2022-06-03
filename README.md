# SSD1680 driver

[![Crates.io](https://img.shields.io/crates/v/ssd1680.svg)](https://crates.io/crates/ssd1680)
[![Docs.rs](https://docs.rs/ssd1680/badge.svg)](https://docs.rs/ssd1680)

SPI (4 wire) driver for the SSD1680 Eink display.

## [Documentation](https://docs.rs/ssd1680)

## [Examples](examples)

Examples are stored in per target directories in ssd1680-examples. cd to your preferred example

`cd ssd1680-examples/stm32f1-examples/`

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
