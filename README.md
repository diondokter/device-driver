# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

> A toolkit to write better device drivers, faster.

Read [the book](https://diondokter.github.io/device-driver/) to learn about how to use the project to build your own device drivers.

## Architecture

This toolkit consists of these parts:

- `device-driver`: The main crate you as the writer of a driver should include in your project.
  It defines a set of types used by the generated code and (by default) reexports the macros.
- `device-driver-generation`: The generation crate contains the device-driver compiler. It takes the tokentree or textual
  inputs and generates the device driver.
- `device-driver-macros`: A small frontend to the generation crate. It can take the dsl token stream or open a text file
  and feed that to the compiler and it outputs the compiler output.
- `device-driver-cli`: A small command line interface that uses the generation crate. It allows you to generate the driver in advance to reduce compile times.
- `dd-manifest-tree`: A small abstraction over json, yaml and toml crates to unify their value types.
- `tests`: A suite of tests that presents input files and compares the known output with the generated output.

## Semver

Anything that can reasonably break user code will warrant a *breaking* semver bump.

The `generation` and `macros` crates are considered internal and so might not be as strict in their semver bumps.
This is mostly to keep them somewhat in line with the version of the main crate.

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
