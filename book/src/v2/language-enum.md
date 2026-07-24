# Enum

Enums work similarly to enums in most languages (like C & Rust) and generates a native enum as the output.

Each property in the node body defines a variant and at least one must be defined.
When a variant doesn't specify a number value, it will be incremented by one from the previous variant or be zero when it's the first variant.

The bit size of the enum is automatically determined based on the variants of the enum and the base type.
Any [field] that uses the enum as conversion type must have the same base type as the enum.

An enum that covers all bit patterns for its bit size can be used for infallible conversion. This is possible by having a variant for each bit pattern or by having a default or catch-all variant.

An enum with a default value will collapse the value into the default if the value is not expressed by any other variant. A catch-all catches the value and retains it. You should prefer using a default value and only use catch-all when you require reflexivity.

{{#include ../gen-docs/mir-shapes/enum.md}}

[field]: ./language-field.md
