# Dsl syntax

> [!CAUTION]
> This doc is written manually. The implementation may differ.
> If it does, then either this doc is wrong or the implementation is wrong.
> In any case, them disagreeing is a bug. Please file an issue!

> [!WARNING]
> While something may be valid to be parsed, it may not be valid as a construct
> and may generate an error deeper down.

Top-level item is _Device_.

- '*' is used to signal 0 or more instances.
- '?' is used to signal 0 or 1 instances.
- '|' is used as an 'or'. One of the options in the chain can be used.
- '( )' is used to group things together.
- Any `keyword` or brackets in the grammer use backticks just like word 'keyword' on this line.

This doesn't map perfectly on the YAML and JSON inputs, but they should be made as close as possible.

_Device_:
> _GlobalConfigList_  
> _ObjectList_

_GlobalConfigList_:
> (`config` `{` _GlobalConfig_* `}`)?

_GlobalConfig_:
> (`type` `DefaultRegisterAccess` `=` _Access_`;`)  
> | (`type` `DefaultFieldAccess` `=` _Access_`;`)  
> | (`type` `DefaultBufferAccess` `=` _Access_`;`)  
> | (`type` `DefaultByteOrder` `=` _ByteOrder_`;`)  
> | (`type` `DefaultBitOrder` `=` _BitOrder_`;`)  
> | (`type` `RegisterAddressType` `=` _IntegerType_`;`)  
> | (`type` `CommandAddressType` `=` _IntegerType_`;`)  
> | (`type` `BufferAddressType` `=` _IntegerType_`;`)  
> | (`type` `NameWordBoundaries` `=` _NameWordBoundaries_`;`)  
> | (`type` `DefmtFeature` `=` _String_`;`)  

_NameWordBoundaries_:
This specifies the input, not the output. Only applies to object and field names.
> [_Boundary_*]  
> | _String_

_ObjectList_:
> (_Object_(`,` _Object_)*`,`?)?

_Object_:
> _Block_  
> | _Register_  
> | _Command_  
> | _Buffer_  
> | _RefObject_  

_RefObject_:
An object that is a copy of another object. Any items in the object are overrides.
> _AttributeList_
> `ref` _IDENTIFIER_ `=` _Object_

_AttributeList_:
> _Attribute_*

_Attribute_:
Used for documentation and conditional compilation
> (`#` `[` `doc` `=` _STRING_`]`)  
> | (`#` `[` `cfg` `(` _ConfigurationPredicate_`)` `]`)  

_Block_:
> _AttributeList_  
> `block` _IDENTIFIER_ `{` _BlockItemList_ _ObjectList_ `}`  

_BlockItemList_:
> _BlockItem_*

_BlockItem_:
> (`const` `ADDRESS_OFFSET` `=` _INTEGER_`;`)  
> | (`const` _Repeat_)  

_Register_:
> _AttributeList_  
> `register` _IDENTIFIER_ `{` _RegisterItemList_ _FieldList_ `}`  

_RegisterItemList_:
> _RegisterItem_*

_RegisterItem_:
> (`type` `Access` `=` _Access_`;`)  
> | (`type` `ByteOrder` `=` _ByteOrder_`;`)  
> | (`type` `BitOrder` `=` _BitOrder_`;`)  
> | (`const` `ADDRESS` `=` _INTEGER_`;`)  
> | (`const` `SIZE_BITS` `=` _INTEGER_`;`)  
> | (`const` `RESET_VALUE` `=` _INTEGER_ | _U8_ARRAY_`;`)  
> | (`const` _Repeat_)  
> | (`const` `ALLOW_BIT_OVERLAP` = _BOOL_`;`)  
> | (`const` `ALLOW_ADDRESS_OVERLAP` = _BOOL_`;`)  

_Access_:
> (`ReadWrite`|`RW`)  
> | (`ReadOnly`|`RO`)  
> | (`WriteOnly`|`WO`)

_ByteOrder_:
> `LE`|`BE`

_BitOrder_:
> `LSB0`|`MSB0`

_FieldList_:
> (_Field_ (`,` _Field_)* `,`?)

_Field_:
> _AttributeList_  
> _IDENTIFIER_`:` _Access_? _BaseType_ _FieldConversion_? `=` _FieldAddress_

_FieldConversion_:
> (`as` `try`? _TYPE_PATH_)  
> | (`as` `try`? `enum` _IDENTIFIER_ `{` _EnumVariantList_`}`)

_EnumVariantList_:
> _EnumVariant_(`,` _EnumVariant_)*`,`?

_EnumVariant_:
> _AttributeList_  
> _IDENTIFIER_ (`=` _EnumValue_)?

_EnumValue_:
> _INTEGER_|`default`|`catch_all`

_FieldAddress_:
> _INTEGER_  
> | (_INTEGER_`..`_INTEGER_)  
> | (_INTEGER_`..=`_INTEGER_)

_BaseType_:
> `bool` | `uint` | `int`

_Command_:
> _AttributeList_  
> `command` _IDENTIFIER_ _CommandValue_?

_CommandValue_:
> (`=` _INTEGER_)  
> | (`{` _CommandItemList_ (`in` `{` _FieldList_ `}` `,`?)? (`out` `{` _FieldList_ `}` `,`?)? `}`)

_CommandItemList_:
> _CommandItem_*

_CommandItem_:
Commands have data going in and out, so they need two separate data field types.
If no in fields, then no data is sent. If no out fields, then no data is returned.
> (`type` `ByteOrder` `=` _ByteOrder_`;`)  
> | (`type` `BitOrder` `=` _BitOrder_`;`)  
> | (`const` `ADDRESS` `=` _INTEGER_`;`)  
> | (`const` `SIZE_BITS_IN` `=` _INTEGER_`;`)  
> | (`const` `SIZE_BITS_OUT` `=` _INTEGER_`;`)  
> | (`const` _Repeat_)  
> | (`const` `ALLOW_BIT_OVERLAP` = _BOOL_`;`)  
> | (`const` `ALLOW_ADDRESS_OVERLAP` = _BOOL_`;`)

_Repeat_:
> `REPEAT` `=` `{` `count` `:` _INTEGER_`,` `stride` `:` _INTEGER_`,`? `}` `;`

_Buffer_:
> _AttributeList_  
> `buffer` _IDENTIFIER_(`:` _Access_)? (`=` _INTEGER_)?

