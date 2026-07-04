# Buffer

Bufers are used to represent a stream of bytes on a device. This could for example be a fifo on a radio.
It's quite a simple construct and thus is limited in configuration options.

It defines an operation on the block it's part of. Buffer functionality is implemented in the runtime through the [BufferOperation](https://docs.rs/device-driver/latest/device_driver/struct.BufferOperation.html) which can be used to read and write from/to the buffer. This operation type also implements the [embedded-io](https://crates.io/crates/embedded-io) traits when the cargo feature is activated on the runtime.

Example usage:

```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write_all(&[0, 1, 2, 3]).unwrap();
let mut buffer = [0; 8];
let len = device.bar().read(&mut buffer).unwrap();
```

{{#include ../gen-docs/mir-shapes/buffer.md}}
