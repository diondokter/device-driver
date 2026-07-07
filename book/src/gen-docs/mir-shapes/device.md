## Example

```ddsl
/// doc comment line
device Example {
    byte-order: LE,
    register-address-type: i32,
    command-address-type: i32,
    buffer-address-type: i32,
    word-boundaries: "bD:0B:_",
    register-address-mode: mapped,

    block node,
    register node,
    command node,
    buffer node,
    fieldset node,
    enum node,
    extern node,
}
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `Type` |
| Supports repeat | `no` |
| Supports basetype | `no` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `yes`, see below |
## Long properties
These properties are specified in the node body.
### byte-order
Sets the default byte order used by fieldsets in this device. This can be overridden per fieldset.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `byte order` => `LE`
### register-address-type
Sets the type used to address the registers in this device.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `integer type` => `i32`
### command-address-type
Sets the type used to address the commands in this device.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `integer type` => `i32`
### buffer-address-type
Sets the type used to address the buffers in this device.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `integer type` => `i32`
### word-boundaries
Sets the word splitting rules for all objects defined in the device.

This option exists to aid in copying names from the datasheet. Those names are often not proper names for types and operations.
So by setting the rules, the compiler can split identifiers into good proper words and then convert them to the required casing.
The splitting is done with `convert_case` using their [`string representation`](https://docs.rs/convert_case/0.10.0/convert_case/enum.Boundary.html#method.defaults_from) for boundaries.

In short, place a colon (`:`) between every boundary. Then each boundary follows the expressed pattern.
For example `aB` will split words when a lower case letter is followed by an upper case letter.
Some symbols are also allowed as boundary, like `-` & `_`.

If not specified, this uses a reasonable default for splitting.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `string` => `"bD:0B:_"`
### register-address-mode
Sets the address mode for registers in this device.

When specified, the registers are assumed to share an address space:
- With the `mapped` option, that address space is a memory-mapped space where if register `A` has address `X` and is `Y` bytes big, then register `B` (if it exists) will have the address `X+Y`.
- With the `indexed` option, that address space has one register per number where if object `A` has address `X`, then object `B` (if it exists) will have the address `X+1`.

If this value is specified, then it permits multi-register reads and writes.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `address mode` => `mapped`
## Possible subnodes
Subnodes of the following types are allowed in the node body.
- [block]
- [register]
- [command]
- [buffer]
- [fieldset]
- [enum]
- [extern]
