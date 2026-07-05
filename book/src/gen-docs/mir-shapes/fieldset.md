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
The size of the fieldset in number of bytes.
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `number` => `8`
### byte-order
The byte order of the fieldset data.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `byte order` => `LE`
### bit-overlap
Allows fields to overlap. This is not allowed by default to prevent copy-paste mistakes.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `allow` => `allow`
## Possible subnodes
Subnodes of the following types are allowed in the node body.
- [field]
