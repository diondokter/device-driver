# Blocks

A block is a collection of other objects. This can be great to e.g. pool related objects together.

Blocks have their own address offset which is applied to all child objects. With this repeated and ref blocks are supported and can be used to great effect.

> [!TIP]
> The generated code has one implicit root block with the name of the device that acts as the entry point of the driver.
> The only difference with other blocks is that it takes ownership of the interface instance and always has address offset 0.

It is accessed as a function on the parent block it's part of.

All objects are generated globally so child objects still need a globally unique name and are not generated in a module.

Example usage:
```rust
// MyDevice is the root block
let mut device = MyDevice::new(DeviceInterface::new());

let mut child_block = device.foo();
child_block.bar().dispatch().unwrap();
// Or in one go
device.foo().bar().dispatch().unwrap();
```

Below are minimal and full examples of how blocks can be defined.
There's one child object defined as example.

- [Blocks](#blocks)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`type` (manifest only)](#type-manifest-only)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)
    - [`address_offset`](#address_offset)
    - [`repeat`](#repeat)
    - [`objects` (manifest only)](#objects-manifest-only)

## DSL

Minimal:
```rust
block Foo {
    buffer Bar = 0,
}
```

Full:
```rust
/// Block description
#[cfg(not(blah))]
block Foo {
    const ADDRESS_OFFSET = 10;
    const REPEAT = {
        count: 2,
        stride: 20,
    };

    buffer Bar = 0,
}
```

## Manifest

Minimal:
```json
"Foo": {
    "type": "block",
    "objects": {
        "Bar": {
            "type": "buffer",
            "address": 0
        }
    }
}
```

Full:
```json
"Foo": {
    "type": "block",
    "cfg": "not(blah)",
    "description": "Block description",
    "address_offset": 10,
    "repeat": {
        "count": 2,
        "stride": 20,
    },
    "objects": {
        "Bar": {
            "type": "buffer",
            "address": 0
        }
    }
}
```

## Required

### `type` (manifest only)

The type of the object.

For blocks this field is a string with the contents `"block"`.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the block.

In the DSL, the normal Rust syntax is used. Just put the attribute on the block definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated block struct and on the function to access the block.

### `address_offset`

The address offset used for all child objects specified as a signed integer.

The offset is applied to all addresses of the children. So when the offset is 5 and a child specifies address 7, then the actual used address will be 12.

If the offset is not specified, it is default 0.

### `repeat`

Repeat the block a number of times at different address offsets.

It is specified with two fields:
- Count: unsigned integer, the amount of times the block is repeated
- Stride: signed integer, the amount the address offset changes per repeat

The calculation is `offset = base_offset + index * stride`.

When the repeat field is present, the function to access a block will have an extra parameter for the index.

### `objects` (manifest only)

A map that contains all the child objects.

For the DSL the children are defined in the block directly.
