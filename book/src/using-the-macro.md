# Using the macro

The macro is the main way of generating a driver. It is defined in the `device-driver-macros` crate which is re-exported in the `device-driver` crate by default. You don't have import the macros crate yourself.

The macro can be used in two forms.

## Inline DSL

The first form is for writing the register definitions using the DSL right in the source of your project.

```rust
device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        // DSL code goes here
    }
)
```

It consists of two parts:
- `device_name`: This will be the name of the root block that will take ownership of the device interface.
  - The name must be provided in PascalCase
  - If you're going to distribute this as the main part of your driver, then it's recommended to use the name
    of the chip this driver is for. For example: 'Lis3dh'
  - If you're going to write a higher level wrapper around it, then it's recommended to name it something
    appropriate for a low level layer. For example: 'Registers' or 'LowLevel'
- `dsl`: This selects the option to write DSL code in the macro

Using the DSL in this way allows for nice error messages and keeps the definitions close to your code.

## Manifest file

The second form uses an external manifest file.

```rust
device_driver::create_device!(
    device_name: MyTestDevice,
    manifest: "driver-manifest.yaml"
)
```

You can provide an absolute path or a relative path to the file. If it's relative, then the base path is the value of the [`CARGO_MANIFEST_DIR` environment variable](https://doc.rust-lang.org/cargo/reference/environment-variables.html). This is the same directory as your `Cargo.toml` is in.

The extension of the file determines which parser is used.

The options are:
- yaml
- json
- toml
- dsl

## Output

> [!tip]
> The generated code is placed exactly where the macro is invoked. This means you can decide to contain everything in its own module. This is recommended to do, but not required.

> [!CAUTION]
> Code in the same module as the generated code is able to access the private API of the generated code. It is discouraged to make use of the private API since it's not considered as part of the SemVer guarantees and it's designed in a way where you shouldn't need to.

> [!NOTE]
> If you feel part of the private API should be stabilized, then please open an issue to discuss it. If you really need to access the private API, consider pinning the exact device-driver versions and make sure to pin the sub crates too, including the generation and the macros crate.

## Optimizing compile times

The device-driver crate has features for turning on the json, yaml and toml parsers. These are enabled by default for your convenience.

Once you've settled on a format, you can optimize the compile times for you and your dependents by disabling the default features and adding back the features you need.

Suggestions:
- When using the DSL (inline or as manifest)
  - `default-features = false`
  - `features = ["dsl"]`
- When using yaml
  - `default-features = false`
  - `features = ["yaml"]`
- When using json
  - `default-features = false`
  - `features = ["json"]`
- When using toml
  - `default-features = false`
  - `features = ["toml"]`

> [!tip]
> With these steps the compile times should be acceptable. However, they can be further optimized by getting rid of the macro alltogether. This is explained in the cli chapter.
