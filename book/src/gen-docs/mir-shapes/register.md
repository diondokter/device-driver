## Example

```ddsl
/// doc comment line
register Example {
    address: 0,
    access: RW,
    address-overlap: allow,
    reset: [12, 34],
    fields: MyFieldset,
}
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `Operation` |
| Supports repeat | `yes` |
| Supports basetype | `no` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `no` |
## Long properties
These properties are specified in the node body.
### address
The address of the register.
```ddsl
// number
address: 0
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
### access
Limits how the register can be accessed. If not specified, the access is `RW`.
```ddsl
// access specifier
access: RW
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### address-overlap
Allows addresses to overlap with other registers. This is not allowed by default to prevent copy-paste mistakes.
```ddsl
// allow
address-overlap: allow
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### reset
Defines the reset value of the register. When performing a write operation, this value loaded in by default.

The value can be expressed in two ways:
- Byte array: No byte order changes are done. The array will be loaded into the fieldset as is.
- Integer: Will be converted to a byte array with the specified byte order.
```ddsl
// [bytes]
reset: [12, 34],
// number
reset: 1234
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### fields
The fieldset that represents the data of the register. This can be a reference to an existing fieldset or a completely new inline fieldset.
```ddsl
// type reference
fields: MyFieldset,
// sub node
fields: fieldset MyFieldSet
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
