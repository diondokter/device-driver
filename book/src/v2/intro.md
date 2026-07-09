# Intro

> [!IMPORTANT]
> We deserve better drivers. Rust has shown that we don't need to stick to old principles and that we as an industry can do better.

Device-driver is a toolkit written in Rust that generates safe, documented interfaces for hardware devices, handling bit-packed registers and device commands through an expressive custom language.

While the Rust language provides many opportunities to improve the way we write drivers, it doesn't mean those are easy to use. There are two issues:
1. Creating good datastructures to represent the driver is hard
2. Writing the definitions along with all its boilerplate takes a lot of thankless work

By using this toolkit, you get both 1 and 2 solved.

Number one is solved by getting the datastructures as part of this toolkit which has seen over 6 years of iteration and improvements. The second issue is solved by using code generation so you only need to take care of the things that make your driver unique.

Together, this delivers a really tight and precise way of authoring your device driver:

```ddsl
// device.ddsl

device MyDevice {
    register-address-type: u8,

    /// This is the Foo register
    register Foo {
        address: 0,
        fields: fieldset _ {
            size-bytes: 1,

            /// This is a bool at bit 0!
            field value0 0 -> bool,
            /// Integrated enum generation
            field value1 3:1 -> _ as enum GeneratedEnum {
                A: _,
                /// Variant B
                B: _,
                C: default _,
            },
            /// This is a 4-bit integer
            field value2 7:4 -> uint,
        }
    },
}
```

```rust
// Generate and include
// `ddc build rust -s device.ddsl -o device.rs --rust-defmt-feature=defmt`
include!("device.rs");

// Or use the macro to compile at build-time
device_driver::compile!(
    options: "--rust-defmt-feature=defmt", // Target options
    manifest: "device.ddsl" // Link to definition
);

let mut device = MyDevice::new(device_interface);
device.foo().write(|reg| reg.set_value_1(GeneratedEnum::B)).unwrap();
```

Instantly we get a nice and familiar API that is well documented.

## The goal

When you're writing a driver, you just want to implement it and be done with it. Most of the time developing a driver is boring and repetitive.

To help you do less of the boring work and to create a higher quality driver at the same time, the goals are:

1. Get a great driver for minimal effort
2. Get a driver that is correct and hard to misuse
   - (assuming the input spec is correct)
3. Get a driver that is well documented
   - (assuming the input spec gives docs)

These goals are met by:
- Using a dense and precise input language
- Having many options to deal with memory layout
- Having analysis steps and great error reporting to decrease the chance of common mistakes
- Separating the interface to the device from the definitions
- Allowing you to put docs on pretty much anything

## How to continue

Simply read the rest of the book!

If you're new, head over to one of the tutorial sections.
If you're looking for a language detail, go see one of the reference sections.

Looking at existing drivers and examples can also be very helpful.

## Future plans

There are many more features this toolkit wishes to support.
To get an up-to-date overview, check out the [issue tracker](https://github.com/diondokter/device-driver/issues).

Zooming out, the plans can be summarized to:
- LSP support for DDSL
- Output to typst/pdf/html for auto docs
- Add support for other languages like C/C++/Zig/TinyGo/MicroPython
- A simple general programming language in DDSL
  - Allow interface definitions
  - Allow simple routines to be implemented in DDSL (for init, sleep, read data)
  - Allow the implemention of statemachines
- Support mixed read-write transactions
- Add string/byte array base types
- Add templates

If you feel strongly about any of this and have ideas/suggestions, feel free to reach out on the appropriate issues or in the matrix chat room.
