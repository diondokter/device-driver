## Example

```ddsl
/// doc comment line
field Example 8:0 RW -> uint as try Foo
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `All` |
| Supports repeat | `yes` |
| Supports basetype | `yes` |
| Supports conversion type | `yes` |
| Supports short properties | `yes`, see below |
| Supports properties | `no` |
| Supports subnodes | `no` |
## Short properties
These properties are specified inline in the node definition and are used without name.
### address
The bit address of the field within the fieldset
```ddsl
// range
address: 8:0,
// number
address: 0
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
### access
Limits how the field can be accessed. If not specified, the access is `RW`.
```ddsl
// access specifier
access: RW
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
