# Buffers

A buffer is used to represent an stream of bytes on a device. This could for example be a fifo for a radio.
It's quite a simple construct and thus is limited in configuration options.

It is accessed as a function on the block it's part of. The function returns a [BufferOperation](https://docs.rs/device-driver/latest/device_driver/struct.BufferOperation.html) which can be used to read and write from/to the buffer. This operation type also implements the [embedded-io](https://crates.io/crates/embedded-io) traits.

Example usage:

```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write_all(&[0, 1, 2, 3]).unwrap();
let mut buffer = [0; 8];
let len = device.bar().read(&mut buffer).unwrap();
```

Below are minimal and full examples of how buffers can be defined.

- [Buffers](#buffers)
  - [DSL](#dsl)
  - [Manifest](#manifest)
  - [Required](#required)
  - [Optional](#optional)

## DSL

Minimal:
```rust
buffer Foo = 5,
```

Full:
```rust
/// A foo buffer
#[cfg(bar)]
buffer Foo: WO = 5,
```

## Manifest

Full:
```json
"Foo": {
    "type": "buffer",
    "cfg": "bar",
    "description": "A foo buffer",
    "access": "WO",
    "address": 5
},
```

Full:
```json
"Foo": {
    "type": "buffer",
    "cfg": "bar",
    "description": "A foo buffer",
    "access": "WO",
    "address": 5
},
```

## Required

## Optional
