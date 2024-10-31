# Device-driver toolkit
[![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

> A toolkit to write better device drivers, faster.

This book aims to guide you to write your own device drivers using the device-driver toolkit.
It is not a replacement of the [docs](https://docs.rs/device-driver) though. The documentation describes all the small details while this book is concerned with more big-picture concepts and the description of the DSL and manifest (JSON, YAML and TOML) inputs.

> [!NOTE]
> Definitions are important!  
> The name `device-driver` conists of two parts:
> - `driver`: Code to enable the use of hardware.
> - `device`: A chip or peripheral you can talk to over a bus.
>
> Examples of good targets for using this toolkit:
> - An I2C accelerometer
> - A SPI radio transceiver
> - A screen/display with parallel bus
>
> The driver is usable in any no-std context and can be made to work with the `embedded-hal` crate or any custom interfaces.

(In theory this toolkit can be used for memory-mapped peripherals too, but there are likely better crates to use for that like `svd2rust` and `chiptool`. The major difference is that this toolkit assumes device interfaces to be fallible.)

## Book overview:

- The intro chapter describes the goal of the toolkit, what it does for you and why you may want to use it instead of writing the driver manually.
- After the intro are chapters about how to generate and then import the driver code into your project. This can be done during compilation through a proc-macro or ahead of time with the CLI and `include!`.
- Next is a chapter about creating a driver interface where you'll see how to implement the right traits so the generated driver can talk with your device.
- Then the actual definition of the driver is covered. These chapters teach what options there are for defining registers, commands, buffers and more using either the DSL or a manifest language like YAML.

The addendum contains more things that mostly provide useful background informations.

> [!CAUTION]
> It's hard to keep book like this up-to-date with reality. Small errors might creep in despite my best effort.  
> If you do find something out of place, missing or simply wrong, please open an issue, even if it's just for a typo! I'd really appreciate it and helps out everyone.
