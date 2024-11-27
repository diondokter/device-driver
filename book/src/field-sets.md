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
  - [Required](#required)
    - [`base`](#base)
    - [`start`, `end` \& address range](#start-end--address-range)
  - [Optional](#optional)
    - [`cfg` or `#[cfg(...)]`](#cfg-or-cfg)
    - [`description` or `#[doc = ""]`](#description-or-doc--)
    - [`access`](#access)
    - [Conversion](#conversion)
      - [To existing type](#to-existing-type)
      - [To generated enum](#to-generated-enum)

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
    },
    "start": 0,
    "end": 8
  }
}
```

## Required

### `base`

The base type denotes the primitve type used to convert the bits in the address range to a value.

Options:
- uint - unsigned integer
- int - two's complement signed integer
- bool - low or high, only available for 1 bit values

The integer options will generate to the smallest signed or unsigned Rust integers that can fit the value. So a 10-bit uint will become a `u16`.

The value is specified as a string in the manifest format and is written 'as is' in the DSL.

### `start`, `end` & address range

Every field must specified the bitrange it covers. The way this is done differs a bit between the DSL and the manifest but boil down to the same.

The DLS uses `= <ADDRESS>` as the syntax. Valid options for the address are:
- Exclusive range: `0..16`
- Inclusive range: `0..=16`
- Single address: `0`
  - Only in combination with bool base types

The manifest has two fields `start` and `end`, both containing unsigned integers:
- The `start` is the starting bit of the field
- The `end` is the exclusive end bit of the field
  - Not required for bool base types

The address must lie fully within the size of the defining object and no fields may overlap unless the defining object has the `AllowBitOverlap` property set to true.

## Optional

### `cfg` or `#[cfg(...)]`

Allows for cfg-gating the command.

In the DSL, the normal Rust syntax is used. Just put the attribute on the field definition. Only one attribute is allowed.

In the manifest it is configured with a string.
The string only defines the inner part: `#[cfg(foo)]` = `"cfg": "foo",`.

> [!WARNING]
> Check the chapter on cfg for more information. The cfg's are not checked by the toolkit and only passed to the generated code and so there are some oddities to be aware of.

### `description` or `#[doc = ""]`

The doc comments for the generated code.

For the DSL, use the normal doc attributes or triple slash `///`.
Multiple attributes get concatenated with a newline (just like normal Rust does).

For the manifest, this is a string.

The description is added as normal doc comments to the generated code. So it supports markdown and all other features you're used to. The description is used on the generated field getter and setter.

### `access`

Overrides the default field access.

Options are: `RW`, `ReadWrite`, `WO`, `WriteOnly`, `RO`, `ReadOnly`.  
They are written 'as is' in the DSL and as a string in the manifest.

If the specified access can do read, a getter is generated with the name of the field. If the specied access do write, a setter is generated with the `set_` prefix followed by the name of the field.

### Conversion

If the base type of a field is an integer, the value can be converted to a further higher level type. There are two options for this:
- Conversion to an existing type
- Conversion to an inline defined enum value

The conversion can be specified as infallible or fallible. When infallible, the field getter will call on the `From<INTEGER>` trait to convert the base value to the conversion value after which the value is returned. When fallible, the field getter will use the `TryFrom<INTEGER>` trait instead and will return the result value from it.

In the DSL the conversion is specified using the `as <TARGET>` or `as try <TARGET>` keywords for the infallible and fallible variants respectively.

The manifest has two possible fields `conversion` and `try_conversion` for the infallible and fallible variants respectively.

#### To existing type

When a type path is given as the DSL `<TARGET>` or as string in the manifest `conversion` field, the conversion will be done using the specified type.

The type path is used as is in the generated code, so you need to make sure that the type is in scope.

Furthermore the type must implement the `From<INTEGER>` or `TryFrom<INTEGER>` traits for the infallible or fallible conversions respectively when the field has read access. When the field has write access, the type must implement the `Into<INTEGER>` trait.

> [!TIP]
> The existing type can also be a enum generated by the toolkit defined in another place by just using the name of that enum.
> 
> This has an added bonus that the toolkit still has the information for accepted input which means it can use the infallible conversion method instead of the `try` fallible one. This creates a nicer and cleaner API.

#### To generated enum

Instead of a custom type, the toolkit can also generate an enum inline.

In the DSL the format for `<TARGET>` is:
```rust
enum Foo {
    A,
    B = 5, // Also supports bit and hex specification
    /// Comment
    C
}
```
The enum is written pretty much as a normal Rust enum including setting the value of every variant and writing docs on every variant. In this example, the number value of `C` would be 6.

The generated enum will have the same docs as the field (if any).

In the manifest, the same enum would be specified like so:
```json
"conversion": {
  "name": "Foo",
  "description": "Enum docs", // In manifest, enum can be separately documented
  "A": null,
  "B": 5,
  "C": {
    "description": "Comment",
    "value": null
  }
}
```

The values for each variant can be the following:
- Empty or null
  - Use auto counting starting at 0 for the first variant and one higher than the previous variant
- Signed integer
  - To manually specify the value
- `default`
  - To specify a default value
  - When the conversion is of a number that doesn't match any variant, the default variant will be returned
  - In DSL specified 'as is'
  - In manifest specified as a string
  - Also implements the `Default` trait for the enum
- `catch_all`
  - Similar to default, but makes the variant contain the raw value (like `Catch(u8)`)
  - When the conversion is of a number that doesn't match any variant, the catch all will be returned with the raw value
  - In DSL specified 'as is'
  - In manifest specified as a string

When an enum contains both a catch all and a default, the catch all value is used to return unknown numbers.

A generated enum can be used infallibly when any of these properties hold:
- Any bitpattern of the field is covered by an enum variant
- The enum has a default value
- The enum has a catch all value
