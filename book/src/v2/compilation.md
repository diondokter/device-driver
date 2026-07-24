# Compilation

Drivers written with device-driver need to be compiled.

There are multiple ways to compile a ddsl manifest:
- CLI, using the `ddc` command
- Rust proc-macro
- Playground on the website

All of these share the same build options. Those are laid out below.
Some ways of compiling have some options preselected. Check the tool specific chapters for that.

## Build options

Build options are split in three:
- General
- Target specific
- Tool specific

The options are parsed using [clap](https://crates.io/crates/clap). For detailed help, use `--help` as an argument and clap will show the help text that's shown below.

Any argument that starts with `unstable` is exempt from semver and thus should not be relied upon unless the version of device-driver is pinned. Their behavior and names may change at any point or the option might be removed too.

### General

The general build options are available everywhere and change how the compiler acts.

The Rust compile *macro* always uses the Rust target, and so there these options don't have a rust subcommand and include the Rust target options.

```
{{#include ../gen-docs/options/compile-help.txt}}
```

### Target - Rust

All Rust target specific features start with `--rust`.

```
{{#include ../gen-docs/options/rust-help.txt}}
```

### Tool - CLI

The cli can be installed using:
```sh
cargo install device-driver-cli
```

In the future more methods of distribution may become available. (If you have experience with this, help is very much wanted!)

Once installed, you can use the compiler using the `ddc` command (device-driver compiler):
```sh
ddc --help
```

#### Commands

```
{{#include ../gen-docs/options/cli-help.txt}}
```

##### Build

The build command is the primary command for users and needs additional information about input and output. The rest of the options are those from the [general](#general) section.

```
{{#include ../gen-docs/options/cli-build-help.txt}}
```

##### Gen-docs

Generates documentation that is used by this book about the compiler from the source.
This command is only available when the CLI is compiled with the `gen-docs` feature flag.

It's not quite intended for normal use.
The output is unstable.

```
{{#include ../gen-docs/options/cli-gen_docs-help.txt}}
```

### Tool - Rust proc-macro

To ease the compilation flow for Rust projects, a proc-macro is available.
When the `macros` feature is activated on the rust runtime `device-driver` crate, the macro will be exported.

```rust
device_driver::compile!(
    options: "--rust-defmt-feature=defmt",
    manifest: "path/to/manifest.ddsl",
);
```

With the options field you can pass the options described above, with the exception that the `rust` command is already given. (It would not make sense to compile to something other than Rust in a proc-macro.) Use `--help` there to see all the specific details.

The generated code is then emitted by the proc-macro and so the driver will be in the place where the macro is called.

For better UX, it's recommended to add the manifest file to the build.rs:
```rust
fn main() {
    println!("cargo:rebuild-if-changed=path/to/manifest.ddsl");
}
```

> [!IMPORTANT]
> Enabling the macro causes the crate to pull in the compiler as a dependency which increases compile times. It's not huge, but it's definitely present.

### Tool - Playground

For the playground, go to [https://device-driver.com/playground](https://device-driver.com/playground).

The playground supports all targets and there's a text field where the options can be specified.
Using `--help` there works and will cause the help text to be printed to the bottom diagnistics panel.
