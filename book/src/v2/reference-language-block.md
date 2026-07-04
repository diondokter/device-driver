# Block

A block helps with grouping objects in your driver and is ultimately a collection of other objects. This can be great to e.g. logically pool related registers together or to repeat them en masse.

Blocks have an address offset which is applied to all child objects. Keep the offset at 0 if you want to use the global addresses for the sub objects.

Blocks are accessed as an operation on the parent block or device it's part of.

All objects are generated globally so child objects still need a globally unique name and are not generated in a module.

Example usage:
```rust
// MyDevice is the root block
let mut device = MyDevice::new(DeviceInterface::new());

// Foo is a block
let mut foo = device.foo();
// Access any operation defined on the block
foo.bar().dispatch()?;

// Or in one go
device.foo().bar().dispatch()?;
```

{{#include ../gen-docs/mir-shapes/block.md}}

[block]: ./reference-language-block.md
[register]: ./reference-language-register.md
[command]: ./reference-language-command.md
[buffer]: ./reference-language-buffer.md
[fieldset]: ./reference-language-fieldset.md
[enum]: ./reference-language-enum.md
[extern]: ./reference-language-extern.md
