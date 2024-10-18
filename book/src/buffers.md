# Buffers

A buffer is used to represent an stream of bytes on a device. This could for example be a fifo for a radio.
It's quite a simple construct and thus is limited in configuration options.

It is accessed as a function on the block it's part of. The function returns a [BufferOperation](https://docs.rs/device-driver/latest/device_driver/struct.BufferOperation.html) which can be used to read and write from/to the buffer. This operation type also implements the [embedded-io](https://crates.io/crates/embedded-io) traits.

Example usage:

```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write_all(&[0, 1, 2, 3]).unwrap();
let mut buffer = [0; 8];
let len = device.bar().read(&mut buffer).unwrap();
```

Below are minimal and full examples of how buffers can be defined.

- [Buffers](#buffers)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`address`](#address)
    - [`type` (manifest only)](#type-manifest-only)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)
    - [`access`](#access)

## DSL

Minimal:
```rust
buffer Foo = 5,
```

Full:
```rust
/// A foo buffer
#[cfg(bar)]
buffer Foo: WO = 5,
```

## Manifest

Full:
```json
"Foo": {
    "type": "buffer",
    "address": 5
},
```

Full:
```json
"Foo": {
    "type": "buffer",
    "cfg": "bar",
    "description": "A foo buffer",
    "access": "WO",
    "address": 5
},
```

## Required

### `address`

The address of the buffer.

Integer value that must fit in the given address type in the global config and can be negative.

### `type` (manifest only)

The type of the object.

For buffers this field is a string with the contents `"buffer"`.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the buffer.

In the DSL, the normal Rust syntax is used. Just put the attribute on the buffer definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated buffer struct and on the function to access the buffer.

### `access`

Overrides the default buffer access.

Options are: `RW`, `ReadWrite`, `WO`, `WriteOnly`, `RO`, `ReadOnly`.  
They are written 'as is' in the DSL and as a string in the manifest.
