# Device driver toolkit #![no_std] [![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

A toolkit to write better device drivers, faster.

See [this](./examples/spi_register_device.rs) example to see how it works. There's also [this OPL2 device driver](https://github.com/diondokter/opl-driver) that is used as reference register implementation for this crate.

You can now also generate an async interface. See [this example](./examples/spi_register_device_async.rs).

Feedback and feature requests are appreciated! Just open an issue on github.

## Example

```rust
// Create our low level device. This holds all the hardware communication definitions
create_low_level_device!(
    /// Our test device
    MyDevice {
        // The types of errors our low level error enum must contain
        errors: [InterfaceError],
        hardware_interface_requirements: { RegisterInterface },
        hardware_interface_capabilities: {
            fn reset(&mut self) -> Result<(), InterfaceError>;
        },
    }
);

// Create a register set for the device
implement_registers!(
    /// The global register set that uses a u8 as register address
    MyDevice.registers<u8> = {
        /// The identification register (which is RO (Read Only))
        #[generate(Debug)] // We want a fancy debug impl
        id(RO, 0, 3) = {
            /// The manufacturer code
            manufacturer: u16 as Manufacturer = RO 0..16, // Cast the raw int to an enum
            /// The version of the chip
            version: u8 = RO 16..20,
            /// The edition of the chip
            edition: u8 = RO 20..24,
        },
        // ....
        // ....
        // ....
    }
);

/// Does some random register things to showcase how everything works
fn run<SPI, CS, RESET>(
    device: &mut MyDevice<ChipInterface<SPI, CS, RESET>>,
) -> Result<(), LowLevelError>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
    RESET: OutputPin,
{
    // We read the manufacturer
    let id = device.registers().id().read()?;

    // Print the id. It is marked with `#[generate(Debug)]`,
    // so it should only show all fields
    println!("{:?}", id);

    // Enable output on pin 0
    device
        .registers()
        .port()
        .write(|w| w.output_0(Bit::Set).mask_0(Bit::Set))?;

    // Disable the irq status bit
    device
        .registers()
        .irq_settings()
        .modify(|_, w| w.irq_status(Bit::Cleared))?;
    }

    Ok(())
}
```

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

- Better error handling. (Suggestions?)
- Adding memory devices support.
- Allow user to specify default register values.
- Create tools for helping create high level layers.
- Allow register specification based on files? (Work together with [bitinfo](https://crates.io/crates/bitinfo)?)

## Stability

This crate is far from stable. But if it works for you, then I see no reason why you couldn't use it already. Only updating to a new version may break stuff and proper Semver will be used.

## Changelog
### 0.4.0 (13-12-22)
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
