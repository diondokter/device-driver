## Shape
> todo: example

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
> todo: description

- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- number: `0`
### access
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- access specifier: `RW`
### address-overlap
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- allow: `allow`
### reset
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- [bytes]: `[12, 34]`
- number: `1234`
### fields
> todo: description

- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- type reference: `MyFieldset`
- sub node: `fieldset MyFieldSet`
