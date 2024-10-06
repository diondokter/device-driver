# Registers

A register is a piece of addressable memory stored on the device.

It is accessed as a function on the block it's part of. The function returns a [`RegisterOperation`](https://docs.rs/device-driver/latest/device_driver/struct.RegisterOperation.html) which can be used to read/write/modify the register.

```rust
let mut device = MyDevice::new(DeviceInterface::new());

device.foo().write(|reg| reg.set_bar(12345)).unwrap();
assert_eq!(device.foo().read().unwrap().bar(), 12345);
```

## DSL

```rust
/// This is the Foo register
register FooRepeated {
    const ADDRESS = 3;
    const SIZE_BITS = 24;
    const REPEAT = {
        count: 4,
        stride: 3,
    };

    /// This is a bool!
    value0: bool = 0,
    value1: uint = 1..16,
    value2: int = 16..24,
}
```
