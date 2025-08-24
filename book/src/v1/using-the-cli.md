# Using the cli

The cli is there to optimize compile times for your driver users. Instead of having to compile the device-driver macros and run them, you can generate the code ahead of time and then `include!` or make a module out of it.

> [!TIP]
> During development using the proc macro will be lots easier since the code generation won't go out of sync with the driver definitions. Then once the development is done, you may want to use the CLI as an optimization.

## Installation

The cli can be installed using cargo:

```bash
cargo install device-driver-cli
```

This always supports all input formats.

## Usage

The CLI is written with clap and has a minimal and simple interface.

To see all options, use:

```bash
device-driver-cli --help
```

To do the code generation three things are required:

- `-m` or `--manifest`: The path to the manifest file
- `-o` or `--output`: The path to the to be generated rust file
- `-d` or `--device-name`: The name the toolkit will use for the generated device. This must be specified in PascalCase

## Using the output

Exactly how you include the generated rust file is up to you. You could generate it into your `/src` folder and declare it a module, which would be nice for Rust analyzer but forces the generated code to be its own module. Or to include it in an existing module you can use the [`include!`](https://doc.rust-lang.org/core/macro.include.html) macro.

However you choose to include it, don't forget to track the file in your git repo.

The generated code still depends on the device-driver crate, but since we don't depend on the proc macro anymore we can turn off the default features. So in your `Cargo.toml` you can now import the toolkit as:

```toml
device-driver = { version = <VERSION>, default-features = false }
```

This makes it so all unused dependencies are gone.
