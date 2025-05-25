//! Reproduction of https://github.com/diondokter/device-driver/issues/89
//! 
//! This should compile and make 'value' not show up

use field_sets::Foo;

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type DefmtFeature = "defmt-03";
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;

            #[cfg(not(test))]
            value_noshow: uint = 0..4,
            value_show: uint = 4..8,
        },
    }
);

#[test]
fn print_foo() {
    assert_eq!(format!("{:?}", Foo::new()), "Foo { value_show: 0 }");
}
