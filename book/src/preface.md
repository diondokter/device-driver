# Device-driver toolkit
[![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

> A toolkit to write better device drivers, faster.

This book aims to guide you to write your own device drivers using the device-driver toolkit.
It is not a replacement of the [docs]((https://docs.rs/device-driver)) though. The documentation describes all the small details while this book is concerned with more big-picture concepts and the description of the DSL and manifest (JSON & YAML) inputs.

- The intro chapter describes the goal of the toolkit, what it does for you and why you may want to use it instead of writing the driver manually.
- After the intro are two chapters about how to generate and then import the driver code into your project. This can be done during compilation through a proc-macro or ahead of time with the CLI and `include!`.
- Next is a chapter about creating a driver interface where you'll see how to implement the right traits so the generated driver can talk with your device.
- Then the actual definition of the driver is covered. These chapters teach what options there are for defining registers, commands, buffers and more using either the DSL or a manifest language like YAML or JSON.
- Lastly the generation output is covered so you know what you can expect from the generated code of your driver.

The addendum contains more things that mostly provide useful background informations.

> [!CAUTION]
> It's hard to keep book like this up-to-date with reality. Small errors might creep in despite my best effort.  
> If you do find something out of place, missing or simply wrong, please open an issue, even if it's just for a typo! I'd really appreciate it and helps out everyone.
