// Compile test only

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;

            // Same name as register
            value: uint as enum Foo {
                A, B, C, D
            } = 0..2,
            // Check that we can still use absolute paths
            value2: uint as X = 2..4,
            // We can use external crates with ::
            value3: uint as ::core::primitive::u8 = 4..6,
        }
    }
);

type X = u8;
