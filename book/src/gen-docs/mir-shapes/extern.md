## Example

```ddsl
/// doc comment line
extern Example -> uint {
    infallible: allow,
    size-bits: 8,
}
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `Type` |
| Supports repeat | `no` |
| Supports basetype | `yes` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `no` |
## Long properties
These properties are specified in the node body.
### infallible
Allows this type to be infallably converted to.
```ddsl
// allow
infallible: allow
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### size-bits
The size of the type in bits.
```ddsl
// number
size-bits: 8
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
