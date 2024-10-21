# Refs

A ref is a copy of another object where parts of that object are overridden with a new value.

For example, you may have two different registers that have the same fields but reside at different addresses. You may not want to use a repeat if they are not logically repeated.

Refs can target registers, commands and blocks. Buffers can't be reffed because they're so simple there's nothing worth overriding. You also can't ref other refs since that would open the gates of hell in the toolkit implementation.

> [!NOTE]
> Using a ref is exactly the same as using the original, just with the new name. The only difference in API is that if the reset value of a field set is overridden, that fieldset gets an extra constructor with which you can initialize it with the overridden reset value.
> 
> The possible overrides are all of the object properties that *don't* specify things about the field set. For example, `size_bits`, `fields`, `byte_order` and more can't be overridden.

Below are minimal and full examples of how refs can be defined. The examples all override a register and its address.


- [Refs](#refs)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
    - [`target` (manifest only)](#target-manifest-only)
    - [`type` (manifest only)](#type-manifest-only)
    - [`override` or `{ .. }`](#override-or---)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)

## DSL

Minimal:
```rust
register Foo {
    const ADDRESS = 3;
    const SIZE_BITS = 16;

    value: uint = 0..16,
},
ref Bar = register Foo {
    const ADDRESS = 5;
},
```

Full:
```rust
register Foo {
    const ADDRESS = 3;
    const SIZE_BITS = 16;

    value: uint = 0..16,
},
/// This is a copy of Foo, but now with address 5!
#[cfg(feature = "bar-enabled")]
ref Bar = register Foo {
    const ADDRESS = 5;
},
```

## Manifest

Minimal:
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
},
"Bar": {
    "type": "ref",
    "target": "Foo",
    "override": {
        "type": "register",
        "address": 3,
    }
}
```

Full:
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
},
"Bar": {
    "type": "ref",
    "target": "Foo",
    "description": "This is a copy of Foo, but now with address 5!",
    "cfg": "feature = \"bar-enabled\"",
    "override": {
        "type": "register",
        "address": 3,
    }
}
```

## Required

### `target` (manifest only)

The (string) name of the reffed object.

### `type` (manifest only)

The type of the object.

For refs this field is a string with the contents `"ref"`.

### `override` or `{ .. }`

Contains the override fields of the ref.

This is formatted as an object normally is, but some fields will be rejected.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the ref.

In the DSL, the normal Rust syntax is used. Just put the attribute on the ref definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated ref struct and on the function to access the ref.
