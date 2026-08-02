## Example

```ddsl
/// doc comment line
enum Example -> uint {
    /// doc comment line
    Any: _,
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
### *any name*
Defines a variant for the enum. The name of the property becomes the variant name.
```ddsl
// auto
*any name*: _,
// number
*any name*: 0,
// default number
*any name*: default 0,
// default auto
*any name*: default _,
// catch-all number
*any name*: catch-all 0,
// catch-all auto
*any name*: catch-all _
```
#### Info
- required: `no`
- multiple allowed: `yes`
- supports doc comments: `yes`
