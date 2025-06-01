//! Having registers with different access specifiers should still compile

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;
            type Access = RW;

            /// X
            value0: bool = 0,
        },
        register Bar {
            const ADDRESS = 1;
            const SIZE_BITS = 8;
            type Access = RO;

            /// X
            value0: bool = 0,
        },
        register Baz {
            const ADDRESS = 2;
            const SIZE_BITS = 8;
            type Access = WO;

            /// X
            value0: bool = 0,
        }
    }
);
