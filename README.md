# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

A toolkit to write better device drivers, faster.

See [this](https://github.com/diondokter/device-driver/blob/master/examples/spi_device.rs) example to see how it works. There's also [this OPL2 device driver](https://github.com/diondokter/opl-driver) that is used as reference register implementation for this crate.

## License

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

## TODO

- Make hardware interfaces async. (Best way? async/await or nb?)
- Make register R and W type functions const. (Depends on BitVec)
- Better error handling. (Suggestions?)
- Adding memory devices support.
- Allow user to specify default register values.
- Create tools for helping create high level layers.
- Allow register specification based on files? (Work together with [bitinfo](https://crates.io/crates/bitinfo)?)

## Stability

This crate is far from stable. But if it works, then I see no reason why you couldn't use it already. Only updating to a new version may break stuff. However, proper Semver will be used.

