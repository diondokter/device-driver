## Shape
> todo: example

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
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `number` => `0`
- `range` => `8:0`
### access
Limits how the field can be accessed. If not specified, the access is `RW`.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `access specifier` => `RW`
