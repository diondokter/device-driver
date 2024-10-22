# Device driver cli [![crates.io](https://img.shields.io/crates/v/device-driver-cli.svg)](https://crates.io/crates/device-driver-cli) [![Documentation](https://docs.rs/device-driver-cli/badge.svg)](https://docs.rs/device-driver-cli)

> A toolkit to write better device drivers, faster.

Read [the book](diondokter.github.io/device-driver) to learn about how to use the project to build your own device drivers.

> [!TIP]
> While actively developing the it's better to use the proc macro. With the CLI you'll have to run it every time you update any definition.

With this command line interface you can generate the code for your device driver ahead of time and `include!` it in your
project. This can save extra dependencies for proc macro and thus save on compile time.

Install with:
```sh
> cargo install device-driver-cli
```

Then check out the options with:
```sh
device-driver-cli --help
```
