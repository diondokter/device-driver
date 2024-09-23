# Intro

- [Intro](#intro)
  - [The old ways](#the-old-ways)
  - [The goal](#the-goal)
  - [Meeting the goals](#meeting-the-goals)
    - [Input](#input)
    - [Device interface](#device-interface)
    - [Docs](#docs)
  - [How to continue](#how-to-continue)

We deserve better drivers. Rust has shown that we don't need to stick to old principles and that we as an industry can do better.

While the Rust language provides many opportunities to improve the way we write drivers, it doesn't mean those are easy to use. There are two issues:
1. Figuring out how to create registers (and more) using the type system well is hard
2. It takes more time to write more elaborate definitions

While there could be a crate that could figure out the first issue, the latter issue would still be there.

By using this toolkit, you get both 1 and 2 solved.

This is done with inspiration from other parts of the ecosystem and 5 years of iteration. It has led to an awesome representation of a device using the type system that is coupled with the ease of code generation.

## The old ways

So a typical driver in C (or a simple driver in Rust) will have some constants or an enum for the registers.

```rust
#[repr(u8)]
pub enum Register {
    Status = 0x00,
    Ctrl0 = 0x08,
    Ctrl1 = 0x09,
    IntCfg = 0x010,
    OutX = 0x20,
    OutY = 0x21,
    OutZ = 0x22
}
```

Then we need a way to read and write the registers. In Rust this is typically done in a struct.

```rust
pub struct Device<Bus> {
    bus: Bus,
}

impl<Bus: I2C> Device<Bus> {
    pub(crate) fn write_register(&mut self, reg: Register, value: u8) -> Result<(), Error> {
        self.bus.write(DEV_ADDRESS, &[reg as u8, value as u8])?;
        Ok(())
    }

    pub(crate) fn read_register(&mut self, reg: Register) -> Result<u8, Error> {
        let mut buf = [0];
        self.bus.write_then_read(DEV_ADDRESS, &[reg as u8], &mut buf)?;
        Ok(buf[0])
    }
}
```

Everything is fine up until now. But now what? All we have are some register addresses and a way to read/write the device.
There are so many open questions:

- How do we model the data in the registers?
  - Provide bitmasks and shift values?
  - Define it in bitfield?
  - Create bespoke types?
- How do we connect our model to a specific register?
  - Don't connect it at all and depend on the user to do the right thing?
  - Maybe make all registers their own type instead using an enum?
  - Something in the middle?
- How do we provide documentation to the user?
- And more...

Most good solutions require you to do a lot of implementation work.

## The goal

When you're writing a driver, you just want to implement it and be done with it.
Most of the time developing a driver is boring and repetitive.

The goals are:

1. Get a great driver for minimal effort
2. Get a driver that is correct and hard to misuse
   - (assuming the input spec is correct)
3. Get a driver that is well documented
   - (assuming the input spec gives docs)

## Meeting the goals

### Input

To meet the goals, this toolkit provides a way to generate a driver based on a dense and precise input language.

There are three options:

- DSL
  - Rust-like code
- YAML
- JSON

For example, defining a register looks like this in the DSL:

```rust
/// This is the Foo register
register Foo {
    const ADDRESS = 0;
    const SIZE_BITS = 24;

    /// This is a bool!
    value0: bool = 0..1,
    /// 15 bits of unsigned integer
    value1: uint = 1..16,
    /// Inline enum generation too
    value2: int as enum MyEnum {
        Val0,
        Val16 = 0x10,
        Reserved = default,
    } = 16..24,
},
```

There are loads more powerful options like bit and byte ordering specification, repeated registers, fallible and infallible type conversion, name normalization to make copying from the datasheet easier and more.

Many correctness checks are done so there's no overlap in register addresses and field bit addresses. (On by default)

This all is carried over to commands and buffers as well.

The exact details are discussed in further chapters.

### Device interface

Of course some register definitions don't magically know how to talk to a device. For that you need to create an interface struct and implement one of the traits.

There are traits for register, command and buffer interfaces, and each of them has an async variant too. If the blocking interfaces are implemented, then the driver can do blocking operations and if the async interfaces are implemented, the driver can do async operations.

More of this is explained in the interface chapter.

### Docs

Everything is or can be documented.

For the definitions, it up to the spec writer to include the documentation. Almost every field can be documented.

The rest of the this toolkit is well documented through the docs.rs docs and this book.

## How to continue

Simply read the rest of the book!

Looking at existing drivers and examples can also be very helpful.
