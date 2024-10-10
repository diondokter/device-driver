# Intro

> [!IMPORTANT]
> We deserve better drivers. Rust has shown that we don't need to stick to old principles and that we as an industry can do better.

While the Rust language provides many opportunities to improve the way we write drivers, it doesn't mean those are easy to use. There are two issues:
1. Creating good datastructures to represent the driver is hard
2. Writing the definitions by hand takes a lot of thankless work

By using this toolkit, you get both 1 and 2 solved.

Number one is solved by getting the datastructures as part of this toolkit which has seen over 5 years of iteration and improvements. The second issue is solved by using code generation so you only need to manually take care of the things that make your driver unique.

Together, this delivers a really tight and precise way of authoring your device driver:

```rust
device_driver::create_device!(
    device_name: MyDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }
        /// This is the Foo register
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;

            /// This is a bool at bit 0!
            value0: bool = 0,
            /// Integrated enum generation
            value1: int as enum GeneratedEnum {
              A,
              /// Variant B
              B,
              C = default,
            } = 1..4,
            /// This is a 4-bit integer
            value2: uint = 4..8,
        },
    }
);

let mut device = MyDevice::new(device_interface);
device.foo().write(|reg| reg.set_value_1(GeneratedEnum::B)).unwrap();
```

Instantly we get a nice and familiar API that is well documented. There's a bunch more features to discover like using YAML as the input and a bunch of analysis steps, so read on!

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
- Having many options do deal with byte and bit ordering
- Having analysis steps to decrease the chance of common mistakes
- Separating the interface to the device from the definitions
- Allowing you to put docs on pretty much anything

## How to continue

Simply read the rest of the book!

Looking at existing drivers and examples can also be very helpful.
