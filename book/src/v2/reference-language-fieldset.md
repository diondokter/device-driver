# Fieldset

A fieldset is a collection of [field]s that make up the data of a [register], [command] input or command output.

Each fieldset defines a type where each of the fields are accessible through operations with the names of the fields.

> [!NOTE]
> As a user you should not have to construct your fieldsets manually in normal use. But it's available to you for special cases.

Example usage:
```rust
use device_driver::Fieldset;

let mut reg = MyFieldSet::ZERO;
reg.set_foo(1234);
let foo = reg.foo();
```

Fieldsets also implement all bitwise operators for easier manipulation. These operations are done on *all* underlying bits, even ones that are not part of a [field].

There's also an `Into` and `From` implementation to byte arrays of the same size of the fieldset.
All possible bitpatterns are legal.

Example usage:
```rust
let all_ones = !MyFieldSet::from([0x00, 0x00]);
let lowest_byte_set = MyFieldSet::from([0xFF, 0x00]);
let lowest_byte_inverted = all_ones ^ lowest_byte_set;
```

{{#include ../gen-docs/mir-shapes/fieldset.md}}

[field]: ./reference-language-field.md
[register]: ./reference-language-register.md
[command]: ./reference-language-command.md
