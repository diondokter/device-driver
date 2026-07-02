# Compilation

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

#### Commands

```
{{#include ../gen-docs/options/cli-help.txt}}
```

##### Build

The build command is the primary command for users and needs additional information about input and output.

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
