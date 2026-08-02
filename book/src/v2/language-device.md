# Device

A device models the target of the driver (typically a chip you can reach over e.g. SPI). A [manifest] can contain multiple devices. A driver will typically contain at least one device.

A device is spiritually the same as a [block], except it can set some configs and serves as the root of the blocks.

Example usage:
```rust
// Create a device by giving it ownership of a compatible interface
let mut device = MyDevice::new(DeviceInterface::new());

// Use the operations defined on the device
device.foo().read()?;

// When supported, start bulk operations on the device (or any block)
use device_driver::Block; // Must import trait
let (foo, bar) = device
    .bulk_read()
    .with(|d| d.foo().plan())
    .with(|d| d.bar().plan())
    .execute()?;
```

{{#include ../gen-docs/mir-shapes/device.md}}

[manifest]: ./language-manifest.md
[block]: ./language-block.md
[register]: ./language-register.md
[command]: ./language-command.md
[buffer]: ./language-buffer.md
[fieldset]: ./language-fieldset.md
[enum]: ./language-enum.md
[extern]: ./language-extern.md
