# Device driver toolkit 

[![crates.io](https://img.shields.io/crates/v/device-driver-cli.svg)](https://crates.io/crates/device-driver-cli)
[![crates.io](https://img.shields.io/crates/d/device-driver-cli.svg)](https://crates.io/crates/device-driver-cli)
[![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver-cli)
[![matrix](https://img.shields.io/matrix/device-driver:matrix.org)](https://matrix.to/#/#device-driver:matrix.org)

> A toolkit to write better device drivers, faster.

Head over to [the website](https://device-driver.com/) to learn about how to use the project to build your own device drivers.

See the readme in root of the repo for more project information.

> [!TIP]
> While actively developing the it's better to use the proc macro. With the CLI you'll have to run it every time you update any definition.

With this command line interface you can generate the code for your device driver ahead of time and `include!` it in your
project. This can save extra dependencies for proc macro and thus save on compile time.

Install with:
```sh
cargo install device-driver-cli

# Or if you need the extended feature set (e.g. for converting formats):
cargo install device-driver-cli --all-features
```

Then check out the options with:
```sh
ddc --help
```
