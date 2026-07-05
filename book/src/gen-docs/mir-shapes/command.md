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
The address of the command
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `number` => `0`
### address-overlap
Allows addresses to overlap with other commands. This is not allowed by default to prevent copy-paste mistakes.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `allow` => `allow`
### fields-in
The fieldset that represents the input data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `type reference` => `MyFieldset`
- `sub node` => `fieldset MyFieldSet`
### fields-out
The fieldset that represents the output data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
#### Allowed expression types
- `type reference` => `MyFieldset`
- `sub node` => `fieldset MyFieldSet`
