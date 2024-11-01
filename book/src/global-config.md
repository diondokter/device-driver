# Global config

The global config exists to house three kinds of configs:

1. Required
2. Defaults
3. Transformations

> [!NOTE]
> A driver can only have one global config.

Below is a short overview for the DSL format and the manifest format of the global config and their defaults (or `_` for no default). The last chapters describe the options in more detail.

> [!TIP]
> Use the available default values to your advantage to cut back having to specify things on each individual register, command or buffer.

- [Global config](#global-config)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`register_address_type`](#register_address_type)
    - [`command_address_type`](#command_address_type)
    - [`buffer_address_type`](#buffer_address_type)
  - [Defaults](#defaults)
    - [`default_register_access`](#default_register_access)
    - [`default_field_access`](#default_field_access)
    - [`default_buffer_access`](#default_buffer_access)
    - [`default_byte_order`](#default_byte_order)
    - [`default_bit_order`](#default_bit_order)
  - [Transformations](#transformations)
    - [`name_word_boundaries`](#name_word_boundaries)
    - [`defmt_feature`](#defmt_feature)

## DSL

```rust
config {
    type DefaultRegisterAccess = RW;
    type DefaultFieldAccess = RW;
    type DefaultBufferAccess = RW;
    type DefaultByteOrder = _;
    type DefaultBitOrder = LSB0;
    type RegisterAddressType = _;
    type CommandAddressType = _;
    type BufferAddressType = _;
    type NameWordBoundaries = [
        Underscore, Hyphen, Space, LowerUpper,
        UpperDigit, DigitUpper, DigitLower,
        LowerDigit, Acronym,
    ];
    type DefmtFeature = "my-feature";
}
```

## Manifest

> [!NOTE]
> Example is written in json, but works for yaml and toml too when literally translated.

```json
"config": {
    "default_register_access": "RW",
    "default_field_access": "RW",
    "default_buffer_access": "RW",
    "default_byte_order": "_",
    "default_bit_order": "LSB0",
    "register_address_type": "_",
    "command_address_type": "_",
    "buffer_address_type": "_",
    "name_word_boundaries": [
        "Underscore", "Hyphen", "Space", "LowerUpper",
        "UpperDigit", "DigitUpper", "DigitLower",
        "LowerDigit", "Acronym"
    ],
    "defmt_feature": "my-feature"
}
```

## Required

### `register_address_type`

Specifies the integer type used to represent the address of a register. It is required once a register has been defined.

The value is a string in manifest form or an integer type in DLS form.

Options are: `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `i64`

### `command_address_type`

Specifies the integer type used to represent the address of a command. It is required once a command has been defined.

The value is a string in manifest form or an integer type in DLS form.

Options are: `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `i64`

### `buffer_address_type`

Specifies the integer type used to represent the address of a buffer. It is required once a buffer has been defined.

The value is a string in manifest form or an integer type in DLS form.

Options are: `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `i64`

## Defaults

### `default_register_access`

Provides a default to the access type of registers. Any register can override this.

The value is a string in manifest form or written 'as is' in the DSL.

Options are: `RW` (default), `ReadWrite`, `RO`, `ReadOnly`, `WO`, `WriteOnly`

### `default_field_access`

Provides a default to the access type of fields. Any field can override this.

The value is a string in manifest form or written 'as is' in the DSL.

Options are: `RW` (default), `ReadWrite`, `RO`, `ReadOnly`, `WO`, `WriteOnly`

### `default_buffer_access`

Provides a default to the access type of buffers. Any buffer can override this.

The value is a string in manifest form or written 'as is' in the DSL.

Options are: `RW` (default), `ReadWrite`, `RO`, `ReadOnly`, `WO`, `WriteOnly`

### `default_byte_order`

Sets the global byte order. This is used for the register and command fieldsets.
Any command or register can override it.

The value is a string in manifest form or written 'as is' in the DSL.

Options are: `LE`, `BE`

### `default_bit_order`

Sets the global bit order. This is used for the register and command fieldsets.
Any command or register can override it.

The value is a string in manifest form or written 'as is' in the DSL.

Options are: `LSB0` (default), `MSB0`

## Transformations

### `name_word_boundaries`

All object, field, enum and enum variant names are converted to the correct casing for where it's used in the generated code. This is because some of them have dual use like the object names which are used as struct names (PascalCase) and function names (snake_case).

This also aids when copying names from datasheets since they're often weird, inconsistent, wrong or all three in regards to casing.

> [!IMPORTANT]
> To do proper casing, it must be known when a new word starts. The transition from one word to the next is called a boundary.

The conversions are done using the `convert_case` crate. With this config option you can specify the boundaries the crate uses to do the conversions.

Options are: `[Boundary]` or `string`

The available boundries can be found in [the docs](https://docs.rs/convert_case/0.6.0/convert_case/enum.Boundary.html) of the crate. The boundary names should be specified as strings in the manifest and 'as is' in the DSL.

The string is converted to an array of boundaries using [this function](https://docs.rs/convert_case/0.6.0/convert_case/enum.Boundary.html#method.list_from) which is a really easy way to define it.

The default value is also provided by the crate from [this function](https://docs.rs/convert_case/0.6.0/convert_case/enum.Boundary.html#method.defaults).

### `defmt_feature`

When defined the generated code will have defmt implementations on the types gated behind the feature configured with this option.
The feature gate looks like: `#[cfg(feature = "<VALUE>")]`.
This allows you, the driver author, to optionally include defmt support.

The value is a string in manifest form and also written as a string in the DSL.
