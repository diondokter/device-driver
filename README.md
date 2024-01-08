# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

A toolkit to write better device drivers, faster.

This crate is a set of traits to represent devices and registers.
You can make use of macros to implement the registers in a convenient way.

With quite some ease you can give users a blocking and an async interface to the device.

Currently you've got these options:

- Macro using rust-like syntax: [example](./device-driver/examples/test-macro-driver.rs)
- Generating from external json file: [example](./device-driver/examples/test-json-driver.rs)
- Generating from external yaml file: [example](./device-driver/examples/test-yaml-driver.rs)

Feedback and feature requests are appreciated! Just open an issue on github.
I want to add more features to this crate, but only once I (or maybe you) need them.

Still wanted:

- Add ability to read multiple registers in one go
- CLI to generate implementation once (better compile times for driver users)

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

## Changelog

### 0.5.2 (08-01-24)

- Async read and modify worked with registers instead of their read and write types

### 0.5.1 (08-01-24)

- Removed accidental dependency that forced std

### 0.5.0 (07-01-24)

- *Breaking*: Built up the crate from the ground up again.
  Now more dependent on the type-system and all macros are proc-macros.
  This makes it a whole lot more maintainable and expandable.
- New register trait
- New device trait
- Almost all old features still supported

### 0.4.1 (13-12-22)
- Accidentally left the async flag on by default for 0.4.0 which caused it not to compile on stable.
### 0.4.0 (13-12-22) (yanked)
- Added async support for the register interfaces. Use the `async` feature flag to activate it.
  When you do, you will have access to the `ll::register_async` module that will generate async code for you.
- Updated dependencies (mainly bitvec to 1.0, which makes this release a technically breaking change)

### 0.3.1 (22-12-21)
- Added docs to low level error ([#14](https://github.com/diondokter/device-driver/pull/10))
### 0.3.0 (02-05-21)
- Added better `Debug` impls to all register `R` that prints the raw value in hex.
  There's now also the option (`#[generate(Debug)]`) to get an even better `Debug` impl that also prints out all the fields,
  but does require all fields to impl `Debug` themselves.
  See ([#10](https://github.com/diondokter/device-driver/pull/10)) to see how it works.
### 0.2.0 (19-04-21)
- All user interaction with a 'W' is now through &mut instead of directly to support more kinds of code structuring ([#7](https://github.com/diondokter/device-driver/pull/7))
