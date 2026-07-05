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
The address of the register.
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `number` => `0`
### access
Limits how the register can be accessed. If not specified, the access is `RW`.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `access specifier` => `RW`
### address-overlap
Allows addresses to overlap with other registers. This is not allowed by default to prevent copy-paste mistakes.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `allow` => `allow`
### reset
Defines the reset value of the register. When performing a write operation, this value loaded in by default.

The value can be expressed in two ways:
- Byte array: No byte order changes are done. The array will be loaded into the fieldset as is.
- Integer: Will be converted to a byte array with the specified byte order.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `[bytes]` => `[12, 34]`
- `number` => `1234`
### fields
The fieldset that represents the data of the register. This can be a reference to an existing fieldset or a completely new inline fieldset.
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `type reference` => `MyFieldset`
- `sub node` => `fieldset MyFieldSet`
