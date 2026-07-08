# Extern

An extern allows users to provide their own custom types and use them as conversion types.

The extern must be available to the generated code with the name directly. No namespacing is applied.

By default extern types can only used fallibly and will have the bit size of the base type.

{{#include ../gen-docs/mir-shapes/extern.md}}
