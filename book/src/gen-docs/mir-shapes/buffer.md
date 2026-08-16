## Example

```ddsl
/// doc comment line
buffer Example {
    access: RW,
    address: 0,
}
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `Operation` |
| Supports repeat | `no` |
| Supports basetype | `no` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `no` |
## Long properties
These properties are specified in the node body.
### access
Limits how the buffer can be accessed. Must be specified unless a `default-access` is set by a parent object.
```ddsl
// access specifier
access: RW
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### address
The address of the buffer
```ddsl
// number
address: 0
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
