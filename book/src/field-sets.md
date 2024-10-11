# Field sets

A field set is a collection of fields that make up the data of a register, command input or command output.

Each field set generates to a struct where each of the fields are accesible through functions with the names of the fields.

A field set can be created using the `new` function and will be initialized with the reset value (or zero if there is no reset value). When it's desired to get an all-zero version of the field set, you can call `new_zero`.  
When a ref object overrides the reset value, the field set will have an extra constructor `new_as_<ref name>` that will use the reset value override for the initial value.

> [!NOTE]
> As a user you should not have to construct your field sets manually in normal use. But it's available to you for special cases.

Example usage:
```rust
let mut reg = MyFieldSet::new();
reg.set_foo(1234);
let foo = reg.foo();
```

Field sets also implement all bitwise operators for easier manipulation. These operations are done on *all* underlying bits, even ones that are not part of a field.

There's also an `Into` and `From` implementation to the smallest byte array that can fit the entire field set.

Example usage:
```rust
let all_ones = !MyFieldSet::new_zero();
let lowest_byte_set = MyFieldSet::from([0xFF, 0x00]);
let lowest_byte_inverted = all_ones ^ lowest_byte_set;
```

Below are minimal and full examples of how fields can be defined. There are three major variants:
- Base type
- Converted to custom type
- Converted to generated enum

The conversions can be fallible or infallible. When the fallible `try` option is used, reading the field will return a result instead of the type directly. For generated enums, even though they might not be generally infallible when converted from their base type, the toolkit uses extra range information to see if it can safely present an infallible interface regardless.

- [Field sets](#field-sets)
  - [DSL](#dsl)
  - [Manifest](#manifest)

## DSL

Simple (base type only):
```rust
foo: uint = 0..5,
bar: bool = 5,
zoof: int = 6..=20,
```

With attributes and access specifier:
```rust
/// Field comment!
#[cfg(blah)]
foo: WO uint = 0..5,
```

With conversion to custom type:
```rust
foo: uint as crate::MyCustomType = 0..16,
bar: int as try crate::MyCustomType2 = 16..32,
```

With conversion to generated enum:
```rust
foo: uint as enum GeneratedEnum {
    A,
    B = 5,
    /// Default value
    C = default,
    D = catch_all,
} = 0..8,
```

## Manifest

Simple (base type only) (json):
```json
{
  "foo": { 
    "base": "uint",
    "start": 0,
    "end": 5
  },
  "bar": { 
    "base": "bool",
    "start": 5,
  },
  "zoof": { 
    "base": "int",
    "start": 6,
    "end": 21
  }
}
```

With attributes and access specifier:
```json
{
  "foo": {
    "cfg": "blah",
    "description": "Field comment!",
    "access": "WO",
    "base": "uint",
    "start": 0,
    "end": 5
  }
}
```

With conversion to custom type:
```json
{
  "foo": {
    "base": "uint",
    "conversion": "crate::MyCustomType",
    "start": 0,
    "end": 16
  },
  "bar": {
    "base": "int",
    "try_conversion": "crate::MyCustomType2",
    "start": 16,
    "end": 32
  }
}
```

With conversion to generated enum:
```json
{
  "foo": {
    "base": "uint",
    "conversion": {
      "name": "GeneratedEnum",
      "A": null,
      "B": 5,
      "C": {
        "description": "Default value",
        "value": "default"
      },
      "D": "catch_all"
    }
  }
}
```
