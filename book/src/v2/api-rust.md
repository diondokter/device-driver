# Rust API

The generated driver API follows the device/block layout of the driver specified in the DDSL source. The [namespacing](./language.html#namespacing) rules are also important to keep in mind.

Any object with the `type` namespace will generate a struct definition. Any object with the `operation` namespace will generate a method on a device or block.

```ddsl
device Foo {
    register Bar { ... }
}
```
This source will generate the following shape of Rust code:
```rust
pub struct Foo { ... }
impl Foo {
    pub fn bar() -> RegisterOperation { ... }
}
```

Even though it's possible to define types like enums inside a device, in the generated code they're always global:

```ddsl
device Foo {
    enum Bar { ... }
}
```
```rust
pub struct Foo { ... }
pub enum Bar { ... }
```

> [!TIP]
> To easily inspect the generated code, visit the [playground](https://device-driver.com/playground)!

## Operations

All operations generate functions that return the various `Operation` types. Consult the rust docs for the exact available API:
- [`RegisterOperation`](https://docs.rs/device-driver/latest/device_driver/struct.RegisterOperation.html) 
- [`CommandOperation`](https://docs.rs/device-driver/latest/device_driver/struct.CommandOperation.html) 
- [`BufferOperation`](https://docs.rs/device-driver/latest/device_driver/struct.BufferOperation.html) 

## Devices/blocks

Devices and blocks are very similar to each other in that they can both contain operations.

A device, however, is the root block. As such it always has an address-offset of 0 and can be constructed with an owned interface value.

_TODO: Describe how to init a device once [#183](https://github.com/diondokter/device-driver/issues/183) is resolved_

Both types implement the [`Block`](https://docs.rs/device-driver/latest/device_driver/trait.Block.html) trait, which exposes the interface for cases where you need raw access to it and which allows you to start bulk operations.

> [!IMPORTANT]
> A bulk operation only has access to the device/block it was started on. If the bulk op needs access to the full device, that means you should probably start it on the device.

## Fieldsets

Fieldsets are generated as structs that have the same byte size as specified in the DDSL source.

Each field in a fieldset gets a getter function if the field can be read and a setter function if the field can be written. The getter uses the name of the field and the setter uses the name too, except it prepends it with `set_`.

_TODO: Update text above once [#183](https://github.com/diondokter/device-driver/issues/183) is resolved_

The [Fieldset](https://docs.rs/device-driver/latest/device_driver/trait.Fieldset.html) trait is implemented on all fieldsets which exposes some runtime metadata and a constant `ZERO` init value.

Fieldsets also implement `Into` & `From` for `[u8; N]`, so they can be converted into byte arrays or constructed from byte arrays, as well as the `Default` trait which initializes a fieldset with all bits set to zero.

Fieldsets can be formatted using the `Debug` implementation or with the `defmt::Format` implementation if the appropriate rust compiler option flag is active.

Lastly, the `And`, `Or`, `Xor` and `Not` operator traits are implemented on the fieldsets which do bitwise operations on all of the bits of the fieldsets (including unused bits).

## Enums

Enum objects generate into normal Rust enums. They take on the repr of the used base type and implement `Into` & `(Try)From` to the base type. `TryFrom` is always implemented and `From` is only implemented when all bit patterns of the base type are covered or if the enum contains a default or catch-all.

If the enum has a default variant, then it will implement the `Default` trait that defaults to the marked variant.

Enums can be formatted using the `Debug` implementation or with the `defmt::Format` implementation if the appropriate rust compiler option flag is active.

## Externs

Extern types are not generated, but they are required to implement `Into` & `(Try)From` to their base type since those are used by the generated code.

If the extern allows [`infallible`](./language-extern.html#infallible) conversion, it's expected the `From` trait is implemented.
