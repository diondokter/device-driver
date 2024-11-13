# Registers

A register is a piece of addressable memory stored on the device.

It is accessed as a function on the block it's part of. The function returns a [`RegisterOperation`](https://docs.rs/device-driver/latest/device_driver/struct.RegisterOperation.html) which can be used to read/write/modify the register.

Example usage:
```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write(|reg| reg.set_bar(12345)).unwrap();
assert_eq!(device.foo().read().unwrap().bar(), 12345);
```

Below are minimal and full examples of how registers can be defined.
Only one field is shown, but more can be added. Details about the fields can be read in their own chapter.

- [Registers](#registers)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`address`](#address)
    - [`size_bits`](#size_bits)
    - [`type` (manifest only)](#type-manifest-only)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)
    - [`access`](#access)
    - [`byte_order`](#byte_order)
    - [`bit_order`](#bit_order)
    - [`reset_value`](#reset_value)
    - [`repeat`](#repeat)
    - [`allow_bit_overlap`](#allow_bit_overlap)
    - [`allow_address_overlap`](#allow_address_overlap)
    - [`fields` (manifest only)](#fields-manifest-only)

## DSL

Minimal:
```rust
register Foo {
    const ADDRESS = 3;
    const SIZE_BITS = 16;

    value: uint = 0..16,
}
```

Full:
```rust
/// Register docs
#[cfg(feature = "bar")]
register Foo {
    type Access = WO;
    type ByteOrder = LE;
    type BitOrder = LSB0;
    const ADDRESS = 3;
    const SIZE_BITS = 16;
    const RESET_VALUE = 0x1234;  // Or [0x34, 0x12]
    const REPEAT = {
        count: 4,
        stride: 2
    };
    const ALLOW_BIT_OVERLAP = false;
    const ALLOW_ADDRESS_OVERLAP = false;

    value: uint = 0..16,
}
```

> [!TIP]
> `type` or `const`, which one is it?  
> It's `type` if it's overriding a global config and `const` if it's not.

## Manifest

> [!NOTE]
> The biggest differences with the DSL are the additional `type` field to specify which type of object this is and the `fields` field that houses all fields.

Minimal (json):
```json
"Foo": {
    "type": "register",
    "address": 3,
    "size_bits": 16,
    "fields": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 16
        }
    }
}
```

Full (json):
```json
"Foo": {
    "type": "register",
    "cfg": "feature = \"foo\"",
    "description": "Register docs",
    "access": "WO",
    "byte_order": "LE",
    "bit_order": "LSB0",
    "address": 3,
    "size_bits": 16,
    "reset_value": 4066, // Or [52, 18] (no hex in json...)
    "repeat": {
        "count": 4,
        "stride": 2
    },
    "allow_bit_overlap": false,
    "allow_address_overlap": false,
    "fields": {
        "value": {
            "base": "uint",
            "start": 0,
            "end": 16
        }
    }
}
```

## Required

### `address`

The address of the register.

Integer value that must fit in the given address type in the global config and can be negative.

### `size_bits`

The size of the register in bits.

Positive integer value. No fields can exceed the size of the register.

### `type` (manifest only)

The type of the object.

For registers this field is a string with the contents `"register"`.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the register.

In the DSL, the normal Rust syntax is used. Just put the attribute on the register definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated register struct and on the function to access the register.

### `access`

Overrides the default register access.

Options are: `RW`, `ReadWrite`, `WO`, `WriteOnly`, `RO`, `ReadOnly`.  
They are written 'as is' in the DSL and as a string in the manifest.

Anything that is not `ReadWrite` will limit the functions you can call for the registers. `.write` is only available when the register has write access, `.read` only when the register has read access and `.modify` only when the register has full access.

> [!NOTE]
> This only affects the capability of a register being read or written.
> It does not affect the `access` specified on the fields.
>
> This means you can have a register you cannot write, but does have setters for one or more fields.  
> That won't be harmful or break things, but might look weird.

### `byte_order`

Overrides the default byte order.

Options are: `LE`, `BE`.  
They are written 'as is' in the DSL and as a string in the manifest.

When the size of a register is > 8 bits (more than one byte), then either the byte order has to be defined globally as a default or the register needs to define it.

### `bit_order`

Overrides the default bit order. If the global config does not define it, it's `LSB0`.

Options are: `LSB0`, `MSB0`.  
They are written 'as is' in the DSL and as a string in the manifest.

### `reset_value`

Defines the reset or default value of the register.

Can be a number or an array of bytes.

> [!WARNING]
> When specified as an array, this must be formatted as the bytes that are returned by the `RegisterInterface` implementation. This means that when the register has little endian byte order, the reset value number `0x1234` would be encoded as `[0x34, 0x12]` in the array form.  
> The same concern is there for the bit order.

It is used in the `.write` function. To reset a register to the default value, it'd look like `.write(|_|())`. When a zero value is desired instead of the default, you can use the `.write_with_zero` function instead.

### `repeat`

Repeat the register a number of times at different addresses.

It is specified with two fields:
- Count: unsigned integer, the amount of times the register is repeated
- Stride: signed integer, the amount the address changes per repeat

The calculation is `address = base_address + index * stride`.

When the repeat field is present, the function to do a register operation will have an extra parameter for the index.

### `allow_bit_overlap`

Allow field addresses to overlap.

This bool value is false by default.

### `allow_address_overlap`

Allow this register to have an address that is equal to another register address.
This calculation is also done for any repeat addresses.

Only exact address matches are checked.

This bool value is false by default.

### `fields` (manifest only)

The fields of the register.

A map where the keys are the names of the fields. All values must be fields.
