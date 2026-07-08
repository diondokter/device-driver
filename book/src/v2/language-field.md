# Field

A field is a slice of bits within a [fieldset].

Each field must specify their bit address and can be limited in access (RW, RO or WO).
All fields must also specify a base type, which is a type that can be converted to and from a bit slice.

This *raw* type is not always desired, and so those types can be converted to enums and externs.
The `try` keyword here will mark the conversion as 'fallible' and is often required when the conversion can fail.

Interaction with the fields from code is done through setters and getters.

Example usage:
```rust
let mut reg = MyFieldSet::ZERO;

reg.set_foo(1234);
let foo = reg.foo();

reg.set_bar(MyEnum::A);
let bar = reg.bar()?;
```

{{#include ../gen-docs/mir-shapes/field.md}}

[fieldset]: ./language-fieldset.md
