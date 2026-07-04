## Shape
> todo: example

| Property | Value |
| --- | --- |
| Identifier namespace | `All` |
| Supports repeat | `no` |
| Supports basetype | `no` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `yes`, see below |
## Long properties
These properties are specified in the node body.
### byte-order
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- byte order: `LE`
### register-address-type
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- integer type: `i32`
### command-address-type
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- integer type: `i32`
### buffer-address-type
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- integer type: `i32`
### word-boundaries
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- string: `"bD:0B:_"`
### register-address-mode
> todo: description

- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- address mode: `mapped`
## Possible subnodes
Subnodes of the following types are allowed in the node body.
- [device]
- [fieldset]
- [enum]
- [extern]
