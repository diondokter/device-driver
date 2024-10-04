# Writing an interface

The device-driver crate and the generated code don't know anything about how to talk to your device.
This means we need to teach it about the interface it has!

Let's first create our device:

```rust,no_run
device_driver::create_device!(
    device_name: MyDevice,
    dsl: {
        // ...
    }
);
```

From this we get our top-level block `MyDevice` which has a `new` function that takes an interface.
So we make our own interface struct and are already able to pass it into the driver.
Let's try it out and imagine we defined a register foo:

```rust,no_run
pub struct MyDeviceInterface<BUS> {
    pub bus: BUS,
}

fn try_out() {
    let bus = init_bus();
    let interface = MyDeviceInterface { bus };

    let mut my_device = MyDevice::new(interface);

    let _ = my_device.foo().read();
    //                      ^^^^ method cannot be called due to unsatisfied trait bounds
    //
    // note: the following trait bounds were not satisfied:
    //       `DeviceInterface: RegisterInterface`
}
```

Oh! We get an error!  
Luckily the compiler already gives us the answer.
The problem is that if we want to do something with registers, we have to know how to read and write them.

> [!IMPORTANT]
> This is done by implementing the right traits on our interface type.
> There's a whole bunch of them: [docs](https://docs.rs/device-driver/latest/device_driver/#traits)

To be able to use registers, you need to implement the `RegisterInterface` trait.
For commands you need `CommandInterface` and for buffers you need `BufferInterface`.

Of each of these is an async version too. Implement those and you're able to use the async version of all operations. They've got the same name as the normal operations, except they end with `_async`. The register `.read()` then becomes `.read_async()`.

Let's make our example complete by implementing the `RegisterInterface`:

```rust,no_run
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

async fn try_out_sync() {
    let bus = init_async_bus(); // Implements the async I2c trait
    let interface = MyDeviceI2cInterface { bus };

    let mut my_device = MyDevice::new(interface);

    let _ = my_device.foo().read_async().await;
}
```

Great! You've now learned how to create an interface type and implement the interface trait you need on it.

Some chips can have multiple interfaces, like both SPI and I2C or SPI and QSPI. In that case you might want to create two different interface types for your driver.

> [!TIP]
> You can make your interface type as complex or as simple as you need. What exactly it will look like depends on your chip and your requirements.
