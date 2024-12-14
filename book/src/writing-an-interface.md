# Writing an interface

> [!IMPORTANT]
> The device-driver crate and the generated code don't know anything about how to talk to your device. This means we need to teach it about the interface it has!

Let's first create our device:

```rust,no_run
device_driver::create_device!(
    device_name: MyDevice,
    dsl: {
        // ...
    }
);
```

This generates a top-level block `MyDevice` which has a `new` function that takes ownership of an interface.
We have to create our own interface type that we can pass into it. This type will implement the logic to communicate with the device.

In this example, let's assume a register 'foo' was defined and see what happens:

```rust
/// Our interface struct that owns the bus.
pub struct MyDeviceInterface<BUS> {
    pub bus: BUS,
}

fn try_out() {
    // Initialize the bus somehow. Your HAL should help you there
    let bus = init_bus();
    // Create our custom interface struct
    let interface = MyDeviceInterface { bus };

    // Create the device driver based on the interface
    let mut my_device = MyDevice::new(interface);

    // Try to read the foo register. This results in an error
    let _ = my_device.foo().read();
    // ERROR:               ^^^^ method cannot be called due to unsatisfied trait bounds
    //
    // note: the following trait bounds were not satisfied:
    //       `DeviceInterface: RegisterInterface`
}
```

This example doesn't compile and outputs an error. Luckily the compiler tells us what's wrong. The problem is that we provided a device interface that doesn't provide a way to read or write registers, but we ask the driver to read a register.

The error tells us the device interface should implement the `RegisterInterface` trait.

> [!IMPORTANT]
> Every kind of operation has its own trait. Find the up-to-date docs of them on [docs.rs](https://docs.rs/device-driver/latest/device_driver/#traits).
>
> There's an interface for register, command and buffer.


Of each of the traits there is an async version too. When implemented the async versions of the operations can be used. They've got the same name as the normal operations, except they end with `_async`. The register `.read()` then becomes `.read_async()`.

Let's make our example complete by implementing the `RegisterInterface`:

```rust
pub struct MyDeviceI2cInterface<BUS> {
    pub bus: BUS,
}

// See the docs of the traits to get more up-to-date information about how and what to impl
impl<BUS: embedded_hal::i2c::I2C> device_driver::RegisterInterface for MyDeviceI2cInterface<BUS> {
    // ...
}

// For the async I2C we can implement the async register interface
impl<BUS: embedded_hal_async::i2c::I2C> device_driver::AsyncRegisterInterface for MyDeviceI2cInterface<BUS> {
    // ...
}

fn try_out_sync() {
    let bus = init_sync_bus(); // Implements the I2c trait
    let interface = MyDeviceI2cInterface { bus };

    let mut my_device = MyDevice::new(interface);

    let _ = my_device.foo().read();
}

async fn try_out_async() {
    let bus = init_async_bus(); // Implements the async I2c trait
    let interface = MyDeviceI2cInterface { bus };

    let mut my_device = MyDevice::new(interface);

    let _ = my_device.foo().read_async().await;
}
```

We've now covered how to create an interface type and implement the interface trait you need on it.

Some chips can have multiple interfaces, like both SPI and I2C or SPI and QSPI. You can choose to support them in one type or make separate types for them. 

> [!TIP]
> You can make your interface type(s) as complex or as simple as you need. It depends on your chip and your requirements what it should look like.
> It is good practice, though, to inform the driver users of this with docs and examples.
