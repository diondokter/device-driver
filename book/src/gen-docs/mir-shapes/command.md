## Example

```ddsl
/// doc comment line
command Example {
    address: 0,
    address-overlap: allow,
    fields-in: MyFieldset,
    fields-out: MyFieldset,
}
```
## Table

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
```ddsl
// number
address: 0
```
#### Info
- required: `yes`
- multiple allowed: `no`
- supports doc comments: `no`
### address-overlap
Allows addresses to overlap with other commands. This is not allowed by default to prevent copy-paste mistakes.
```ddsl
// allow
address-overlap: allow
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### fields-in
The fieldset that represents the input data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.
```ddsl
// type reference
fields-in: MyFieldset,
// sub node
fields-in: fieldset MyFieldSet
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
### fields-out
The fieldset that represents the output data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.
```ddsl
// type reference
fields-out: MyFieldset,
// sub node
fields-out: fieldset MyFieldSet
```
#### Info
- required: `no`
- multiple allowed: `no`
- supports doc comments: `no`
