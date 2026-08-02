# Register

A register is a singular piece of addressable memory stored on the device that can be written and/or read.

It defines an operation on the block it's part of. Register functionality is implemented in the runtime through the [`RegisterOperation`](https://docs.rs/device-driver/latest/device_driver/struct.RegisterOperation.html) which can be used to read/write/modify the register.

Example usage:
```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write(|reg| reg.set_bar(12345))?;
assert_eq!(device.foo().read()?.bar(), 12345);
```

{{#include ../gen-docs/mir-shapes/register.md}}
