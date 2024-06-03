# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

A toolkit to write better device drivers, faster.

This crate is a set of traits to represent devices, registers and commands.
You can make use of macros to implement the registers and commands in a convenient way.

With quite some ease you can give users a blocking and an async interface to the device.

Currently you've got these options:

- Macro using rust-like syntax: [example][macro_example]
- Generating from [external json file][json_file]: [example][json_example]
- Generating from [external yaml file][yaml_file]: [example][yaml_example]

Feedback and feature requests are appreciated! Just open an issue on github.
I want to add more features to this crate, but only once I (or maybe you) need them.

I realise the documentation is a bit lacking, but the examples should be pretty straightforward.
For a real driver example using this crate you can also look at: https://github.com/diondokter/s2lp

Still wanted:

- Add ability to read multiple registers in one go
- CLI to generate implementation once (better compile times for driver users)

***Note**: Until https://github.com/illicitonion/num_enum/pull/77 gets merged, you'll need to import the num_enum in the crate where the device-driver macro is used. This needs to be added to the `Cargo.toml`. Trying to import the re-export from device-driver is not enough.* 

## Architecture

This crate consists of three parts:

- `device-driver`: The main crate that re-exports everything needed. It defines a set of traits
  the represent a device and a register. The device traits are to be implemented by the
  driver crate to let the system know how registers are read and written.
  This all is done with the register trait which contains all information needed about the register.
  One could implement the registers and their fields manually, but this crate provides very nice
  functionality to generate all that.
- `device-driver-generation`: This is as it were the backend. It has definitions of a sort of intermediate
  representation of a device and its registers and fields. From there it can generate rust
  code that implements the aforementioned device-driver traits.
  It also provides custom `serde` deserialization routines for various types to make them
  conveniently expressable.
- `device-driver-macros`: Contains all of the proc macros. One macro takes a path to a definitions file.
  It then uses serde to deserialize it into the 'generation' representation. The other macro defines a
  custom Rust syntax and turns that into the 'generation' representation. Both then generate the
  implementation of the registers and their fields.

## Semver

Anything that can reasonably break user code will warrant a *breaking* semver bump.
This includes the macro format and file register definition formats.

The `generation` and `macros` crates are considered internal and so might not be as strict in their semver bumps. This is mostly to keep them somewhat in line with the version of the main crate.

If you depend on these crates directly, please let me know! If I know those have direct users, I will be stricter with the versions.

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

### 0.7.0 (unreleased)

- *Breaking*: Improved the API for dispatching commands
- Added buffer support
- Added strict mode conversion. This makes the types require `From<primitive>` instead of `TryFrom<primitive>`.
  But reading the register field is then not a result.
- Added byte order option to registers so they can be read and stored as little endian. (When not specified, it still defaults to big endian)

### 0.6.0 (26-05-24)

- *Breaking*: Renamed the macros so they don't include the word 'register'
- Added a way to define macros

### 0.5.3 (08-01-24)

- Improved readme with hard links and more explanations
- Generated device register functions are now also documented

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


[yaml_file]: https://github.com/diondokter/device-driver/blob/master/test-files/yaml_syntax.yaml
[json_file]: https://github.com/diondokter/device-driver/blob/master/test-files/json_syntax.json
[macro_example]: https://github.com/diondokter/device-driver/blob/master/device-driver/examples/test-macro-driver.rs
[json_example]: https://github.com/diondokter/device-driver/blob/master/device-driver/examples/test-json-driver.rs
[yaml_example]: https://github.com/diondokter/device-driver/blob/master/device-driver/examples/test-yaml-driver.rs