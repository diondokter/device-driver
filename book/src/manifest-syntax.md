# Manifest syntax

> [!CAUTION]
> This doc is written manually. The implementation may differ.
> If it does, then either this doc is wrong or the implementation is wrong.
> In any case, them disagreeing is a bug. Please file an issue!

> [!WARNING]
> While something may be valid to be parsed, it may not be valid as a construct
> and may generate an error deeper down.

Top-level item is _Device_.

Anything marked like _this_ denotes its own type specification.

These are the pre-defined types:
- bool
- uint
- int
- float
- string
- array
  - Using `[]` brackets.
  - If inner types are restricted, then signaled as e.g. `[float]`
- map
  - Using `{}` brackets.
  - The keys are always text/string.
  - Restrictions can be signaled as required by `?`
  - Restriction syntax: `{ foo?, bar?: float, xen: bool, *: bool }
    - Optional field `foo` without type restriction
    - Optional field `bar` with float restriction
    - Required field `xen` with bool restriction
    - `0..N` fields with any name with bool restriction

Further restriction can be denoted using `oneof()`, for example: `int oneof(1, 2, 3, 4)` or `oneof(bool, int)`

_Device_:
The key of the object will become the name of it
```
{
    config?: _GlobalConfig_,
    *: _Object_
}
```

_GlobalConfig_:
```
{
    default_register_access?: _Access_,
    default_field_access?: _Access_,
    default_buffer_access?: _Access_,
    default_byte_order?: _ByteOrder_,
    default_bit_order?: _BitOrder_,
    register_address_type?: _IntegerType_,
    command_address_type?: _IntegerType_,
    buffer_address_type?: _IntegerType_,
    name_word_boundaries?: _NameWordBoundaries_
    defmt_feature?: string
}
```

_Access_:
```
string oneof("ReadWrite", "RW", "ReadOnly", "RO", "WriteOnly", "WO")
```

_ByteOrder_:
```
string oneof("LE", "BE")
```

_BitOrder_:
```
string oneof("LSB0", "MSB0")
```

_IntegerType_:
```
string oneof("u8", "u16", "u32", "i8", "i16", "i32", "i64")
```

_NameWordBoundaries_:
```
oneof([_Boundary_], string)
```

_Object_:
```
oneof(
  _Block_,
  _Register_,
  _Command_,
  _Buffer_,
  _RefObject_
)
```

_RefObject_:
```
{
    type: string oneof("ref"),
    cfg?: string,
    description?: string,
    target: string,
    override: _Object_,
}
```

_Block_:
```
{
    type: string oneof("block"),
    cfg?: string,
    description?: string,
    address_offset?: int,
    repeat?: _Repeat_,
    objects?: {
        *: _Object_
    }
}
```

_Repeat_:
```
{
    count: uint,
    stride: int
}
```

_Register_:
```
{
    type: string oneof("register"),
    cfg?: string,
    description?: string,
    access?: _Access_,
    byte_order?: _ByteOrder_,
    bit_order?: _BitOrder_,
    address: int,
    size_bits: int,
    reset_value?: oneof(int, [uint]),
    repeat?: _Repeat_,
    allow_bit_overlap?: bool,
    allow_address_overlap?: bool,
    fields?: {
        *: _Field_
    }
}
```

_Field_:
```
{
    cfg?: string,
    description?: string,
    access?: _Access_,
    base: _BaseType_,
    conversion?: _FieldConversion_,
    try_conversion?: _FieldConversion_,
    start: int,
    end?: int,
}
```

_BaseType_:
```
string oneof("bool", "int", "uint")
```

_FieldConversion_:
```
oneof(
    string,
    {
        name: string,
        description?: string,
        *: _EnumVariant_
    }
)
```

_EnumVariant_:
```
oneof(
    _EnumValue_,
    {
        cfg?: string,
        description?: string,
        value?: _EnumValue_
    }
)
```

_EnumValue_:
```
oneof(
    null, int, string oneof("default", "catch_all")
)
```

_Command_:
```
{
    type: string oneof("command"),
    cfg?: string,
    description?: string,
    byte_order?: _ByteOrder_,
    bit_order?: _BitOrder_,
    address: int,
    repeat?: _Repeat_,
    allow_bit_overlap?: bool,
    allow_address_overlap?: bool,
    size_bits_in?: int,
    fields_in?: {
        *: _Field_
    },
    size_bits_out?: int,
    fields_out?: {
        *: _Field_
    },
}
```

_Buffer_:
```
{
    type: string oneof("buffer"),
    cfg?: string,
    description?: string,
    access?: _Access_,
    address: int,
}
```
