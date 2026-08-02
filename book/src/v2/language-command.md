# Command

A command is a call to do something. This can be to e.g. change the chip state, do an RPC-like call or to start a radio transmission.

It defines an operation on the block it's part of. Command functionality is implemented in the runtime through the [`CommandOperation`](https://docs.rs/device-driver/latest/device_driver/struct.CommandOperation.html) which can be used to dispatch the command.

> [!TIP]
> While registers could be modelled as a command, this would be spiritually wrong.
> A device is supposed to *do* something when a command is dispatched.
> It can do something on its own or based the input data. And when the action is done there may be an output.
>
> Examples would be starting radio transmission or putting the device to sleep.

Example usage:
```rust
let mut device = MyDevice::new(DeviceInterface::new());

// Dispatch the foo command
device.foo().dispatch()?;

// Commands carry data when in and/or out fields are specified
let result = device.bar().dispatch(|data| data.set_val(1234))?;
assert_eq!(result.quux(), true);
```

{{#include ../gen-docs/mir-shapes/command.md}}
