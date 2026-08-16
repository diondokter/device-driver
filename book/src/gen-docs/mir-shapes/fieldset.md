## Example

```ddsl
/// doc comment line
fieldset Example {
    size-bytes: 8,
    byte-order: LE,
    bit-overlap: allow,
    default-access: RW,

    field node,
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
### size-bytes
The size of the fieldset in number of bytes.
```ddsl
// number
size-bytes: 8
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
### byte-order
The byte order of the fieldset data.
```ddsl
// byte order
byte-order: LE
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### bit-overlap
Allows fields to overlap. This is not allowed by default to prevent copy-paste mistakes.
```ddsl
// allow
bit-overlap: allow
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### default-access
When set, all subobjects use this value as their access value (unless overridden) and don't require an access specifier anymore
```ddsl
// access specifier
default-access: RW
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
## Possible subnodes
Subnodes of the following types are allowed in the node body.
- [field]
