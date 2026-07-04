## Shape
> todo: example

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
> todo: description

- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- number: `8`
### byte-order
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- byte order: `LE`
### bit-overlap
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- allow: `allow`
## Possible subnodes
Subnodes of the following types are allowed in the node body.
- [field]
