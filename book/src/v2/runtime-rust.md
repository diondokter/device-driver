# Rust runtime

The runtime for rust is the crate called [`device-driver`](https://crates.io/crates/device-driver) and is available on crates.io.

Any rust driver should include the crate as a dependency in the `Cargo.toml`.

The documentation can be found on [docs.rs](https://docs.rs/device-driver/latest/device_driver/).

## Operations

The generated Rust code will make use of the various operation types. These are the main engine of getting your driver to do something.

An operation takes data from the driver and turns them into action. For example, the register operation is used when interacting with registers:

```rust
let mut device = MyDevice::new(interface);

// Get a register operation from the device
// This borrows the interface from the device
let mut operation = device.foo_register();

// Now use the operation, for example to read
let foo_fieldset = operation.read()?;
// Or to write
operation.write(|reg| reg.set_bar(12))?;

// Typically though, the operation is used transparently as it's cleaner
device.foo_register().read()?;
```

There are a bunch of different functions you can call on the operations. Check them out in the documentation.

## Interfaces

Driver definition and interface definition decoupled.

Currently you define the driver in the DDSL code and the interface in Rust. Hopefully in the future the interfaces can be defined in DDSL too.

An interface determines *how* we talk with the device.

The interface always needs to be passed in when a driver instance is created. Depending on what traits are implemented on the interface object, different operation functions are able to be called.

There are different traits for each of the operation types.

For example, if the interface implements the register interface trait, then register operations can read and write. But only the blocking functions will work. For async functions there's the async register interface trait.

So if you find something is not able to be called, check the trait implementations of your interface.

## Bulk operations

Bulk operations are available for registers when the [`register-address-mode`](./language-manifest.md#register-address-mode) is set.

When available, bulk operations must be planned. Start a bulk operation by calling one of the multi-functions on the device (or any block):

```rust
let (foo, bar) = device
    // Start a bulk read
    .multi_read()
    // Plan to read the foo register
    .with(|d| d.foo().plan())
    // Plan to read the bar register after that
    .with(|d| d.bar().plan())
    // Perform the bulk read
    .execute()?;
```
The runtime checks if the plan is allowed. Registers must follow each other up according to the address mode rules.

Bulk operations are possible for repeated registers too.
