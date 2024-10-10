# Commands

A command is a call to do something. This can be to e.g. change the chip state, do an RPC-like call or to start a radio transmission.

It is accessed as a function on the block it's part of. The function returns a [`CommandOperation`](https://docs.rs/device-driver/latest/device_driver/struct.CommandOperation.html) which can be used to dispatch the command.

Example usage:
```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().dispatch().unwrap();
// Commands can carry data too
let result = device.bar().dispatch(|data| data.set_val(1234)).unwrap();
assert_eq!(result.xeno(), true);
```

Below are minimal and full examples of how commands can be defined.
Only one field is shown, but more can be added. Details about the fields can be read in their own chapter.

> [!NOTE]
> A command can have only input, only output, both input and output, or no fields.
> - When input fields are defined, the dispatch function will have a closure parameter to set up the input value.
> - When output fields are defined, the dispatch function return the data that was read back from the device.

- [Commands](#commands)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`address`](#address)
    - [`size_bits_in` \& `size_bits_out`](#size_bits_in--size_bits_out)
    - [`type` (manifest only)](#type-manifest-only)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)
    - [`byte_order`](#byte_order)
    - [`bit_order`](#bit_order)
    - [`repeat`](#repeat)
    - [`allow_bit_overlap`](#allow_bit_overlap)
    - [`allow_address_overlap`](#allow_address_overlap)
    - [`in` (dsl) or `fields_in` (manifest)](#in-dsl-or-fields_in-manifest)
    - [`out` (dsl) or `fields_out` (manifest)](#out-dsl-or-fields_out-manifest)

## DSL

Minimal without fields (with address 5):
```rust
command Foo = 5,
```

Minimal with in and out fields:
```rust
command Foo {
    const ADDRESS = 5;
    const SIZE_BITS_IN = 8;
    const SIZE_BITS_OUT = 16;

    in {
        value: uint = 0..8,
    },
    out {
        value: uint = 0..16,
    }
},
```

Full:
```rust
/// Foo docs
#[cfg(feature = "blah")]
command Foo {
    type ByteOrder = LE;
    type BitOrder = LSB0;
    const ADDRESS = 5;
    const SIZE_BITS_IN = 8;
    const SIZE_BITS_OUT = 16;
    const REPEAT = {
        count: 4,
        stride: 2
    };
    const ALLOW_BIT_OVERLAP = false;
    const ALLOW_ADDRESS_OVERLAP = false;

    in {
        value: uint = 0..8,
    },
    out {
        value: uint = 0..16,
    }
},
```

> [!TIP]
> `type` or `const`, which one is it?  
> It's `type` if it's overriding a global config and `const` if it's not.

## Manifest

> [!NOTE]
> The biggest difference with the DSL is the additional `type` field to specify which type of object this is and the absence of the super short hand.

Minimal with no fields (json):
```json
"Foo": {
    "type": "command",
    "address": 5
}
```

Minimal (json):
```json
"Foo": {
    "type": "command",
    "address": 5,
    "size_bits_in": 8,
    "fields_in": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 8
        }
    },
    "size_bits_out": 16,
    "fields_out": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 16
        }
    },
}
```

Full (json):
```json
"Foo": {
    "type": "command",
    "cfg": "feature = \"blah\"",
    "description": "Foo docs",
    "byte_order": "LE",
    "bit_order": "LSB0",
    "address": 5,
    "repeat": {
        "count": 4,
        "stride": 2
    },
    "allow_bit_overlap": false,
    "allow_address_overlap": false,
    "size_bits_in": 8,
    "fields_in": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 8
        }
    },
    "size_bits_out": 16,
    "fields_out": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 16
        }
    },
}
```

## Required

### `address`

The address of the command.

Integer value that must fit in the given address type in the global config and can be negative.

### `size_bits_in` & `size_bits_out`

The size of the command in bits for their respective field sets.

Positive integer value. No fields can exceed the specified size.

Only required when their respective field sets are defined.

### `type` (manifest only)

The type of the object.

For commands this field is a string with the contents `"command"`.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the command.

In the DSL, the normal Rust syntax is used. Just put the attribute on the command definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated command input and output structs and on the function to access the command.

### `byte_order`

Overrides the default byte order.

Options are: `LE`, `BE`.  
They are written 'as is' in the DSL and as a string in the manifest.

When the size of a command input or output is > 8 bits (more than one byte), then either the byte order has to be defined globally as a default or the command needs to define it.

The value is applied to both the input and output fieldsets.

### `bit_order`

Overrides the default bit order. If the global config does not define it, it's `LSB0`.

Options are: `LSB0`, `MSB0`.  
They are written 'as is' in the DSL and as a string in the manifest.

The value is applied to both the input and output fieldsets.

### `repeat`

Repeat the command a number of times at different addresses.

It is specified with two fields:
- Count: unsigned integer, the amount of times the command is repeated
- Stride: signed integer, the amount the address changes per repeat

The calculation is `address = base_address + index * stride`.

When the repeat field is present, the function to do a command operation will have an extra parameter for the index.

### `allow_bit_overlap`

Allow field addresses to overlap.

This bool value is false by default.

### `allow_address_overlap`

Allow this command to have an address that is equal to another command address.
This calculation is also done for any repeat addresses.

Only exact address matches are checked.

This bool value is false by default.

### `in` (dsl) or `fields_in` (manifest)

The input fields of the command.

- For the dsl, a list of fields.
- For manifest, a map where the keys are the names of the fields All values must be fields.

### `out` (dsl) or `fields_out` (manifest)

The output fields of the command.

- For the dsl, a list of fields.
- For manifest, a map where the keys are the names of the fields All values must be fields.
