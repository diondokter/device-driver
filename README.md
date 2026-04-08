# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

> A toolkit to write better device drivers, faster.

Read [the book](https://diondokter.github.io/device-driver/) to learn about how to use the project to build your own device drivers.

## Versions

- v2: https://github.com/diondokter/device-driver
- v1: https://github.com/diondokter/device-driver/tree/v1

## Architecture

This toolkit consists of these parts:

- `device-driver`: The main crate you as the writer of a driver should include in your project.
  It is the runtime used by the generated code and offers an optional macro to easily compile DDSL code in your Rust project.
- `compiler`: The set of crates that form the compiler.
  - `device-driver-cli`: The source for the `ddc` binary, the traditional compiler executable for DDSL.
  - `device-driver-macros`: The compiler in Rust macro form.
  - `device-driver-wasm`: The compiler in wasm-bindgen form.
- `website`: The source of the [device-driver.com](https://device-driver.com) website.
- `tests`: A suite of tests that presents input files and compares the known output with the generated output.

Of these, only the `device-driver` and `device-driver-cli` crates are considered public.

## Semver

Anything that can reasonably break user code will warrant a *breaking* semver bump.
This only holds if the user is only consuming 'public' device-driver code.

## License

Code licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

All non-code work, unless indicated otherwise, is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), to be attributed to Dion Dokter & device-driver contributors.

All trademarks are owned by Dion Dokter.

## Contribution

Unless you explicitly state otherwise, any code contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Similarly, any non-code contribution shall be licenced under CC BY 4.0 and any trademark ownership shall be transferred to Dion Dokter.

Contributions must be in accordance with the notices in [CONTRIBUTING.md](CONTRIBUTING.md).
