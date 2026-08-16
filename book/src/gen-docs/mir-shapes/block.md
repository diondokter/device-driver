## Example

```ddsl
/// doc comment line
block Example[8 stride 4] {
    address-offset: 0,
    default-access: RW,

    block node,
    register node,
    command node,
    buffer node,
    fieldset node,
    enum node,
    extern node,
}
```
## Table

| Property | Value |
| --- | --- |
| Identifier namespace | `All` |
| Supports repeat | `yes` |
| Supports basetype | `no` |
| Supports conversion type | `no` |
| Supports short properties | `no` |
| Supports properties | `yes`, see below |
| Supports subnodes | `yes`, see below |
## Long properties
These properties are specified in the node body.
### address-offset
Defines the address offset of this block. All objects in the block are relative to the block.
For example, a block with an address offset of 10 which has a register at address 5, will have defined the register at address 15.
If this is not desired, then keep the address offset at 0.
```ddsl
// number
address-offset: 0
```
#### Info
- required: `yes`
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
- [block]
- [register]
- [command]
- [buffer]
- [fieldset]
- [enum]
- [extern]
