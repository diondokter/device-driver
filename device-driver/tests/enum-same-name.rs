// Compile test only

device_driver::create_device!(
    kdl: "
        device MyTestDevice {
            register-address-type u8
            register Foo {
                address 0
                fields size-bits=8 {
                    (uint:Foo)value @1:0 {
                        A
                        B
                        C
                        D
                    }
                    (uint:crate::X)value2 @3:2
                    (uint:::core::primitive::u8)value3 @5:4
                }
            }
        }
    "
);

type X = u8;
