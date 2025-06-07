---
[package]
edition = "2024"
[dependencies]
device-driver = { path="../../../device-driver" }
---
fn main() {}

/// Root block of the Device driver

#[derive(Debug)]
pub struct Device<I> {
    pub(crate) interface: I,

    #[doc(hidden)]
    base_address: u8,
}

impl<I> Device<I> {
    /// Create a new instance of the block based on device interface
    pub const fn new(interface: I) -> Self {
        Self {
            interface,
            base_address: 0,
        }
    }

    /// A reference to the interface used to communicate with the device
    pub(crate) fn interface(&mut self) -> &mut I {
        &mut self.interface
    }

    /// Read all readable register values in this block from the device.
    /// The callback is called for each of them.
    /// Any registers in child blocks are not included.
    ///
    /// The callback has three arguments:
    ///
    /// - The address of the register
    /// - The name of the register (with index for repeated registers)
    /// - The read value from the register
    ///
    /// This is useful for e.g. debug printing all values.
    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
    /// the lies within so it can be printed without matching on it.
    pub fn read_all_registers(
        &mut self,
        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
    ) -> Result<(), I::Error>
    where
        I: ::device_driver::RegisterInterface<AddressType = u8>,
    {
        Ok(())
    }

    /// Read all readable register values in this block from the device.
    /// The callback is called for each of them.
    /// Any registers in child blocks are not included.
    ///
    /// The callback has three arguments:
    ///
    /// - The address of the register
    /// - The name of the register (with index for repeated registers)
    /// - The read value from the register
    ///
    /// This is useful for e.g. debug printing all values.
    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
    /// the lies within so it can be printed without matching on it.
    pub async fn read_all_registers_async(
        &mut self,
        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
    ) -> Result<(), I::Error>
    where
        I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
    {
        Ok(())
    }

    pub fn foo(
        &mut self,
    ) -> ::device_driver::CommandOperation<'_, I, u8, field_sets::FooFieldsIn, ()> {
        let address = self.base_address + 0;

        ::device_driver::CommandOperation::<'_, I, u8, field_sets::FooFieldsIn, ()>::new(
            self.interface(),
            address as u8,
        )
    }
}

/// Module containing the generated fieldsets of the registers and commands
pub mod field_sets {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct FooFieldsIn {
        /// The internal bits
        bits: [u8; 3],
    }

    impl ::device_driver::FieldSet for FooFieldsIn {
        const SIZE_BITS: u32 = 24;
        fn new_with_zero() -> Self {
            Self::new_zero()
        }
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }

    impl FooFieldsIn {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [0, 0, 0] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 3] }
        }

        ///Read the `value` field of the register.
        ///

        pub fn value(&self) -> u32 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u32, ::device_driver::ops::LE>(&self.bits, 0, 24)
            };

            raw
        }

        ///Write the `value` field of the register.
        ///

        pub fn set_value(&mut self, value: u32) {
            let raw = value;

            unsafe {
                ::device_driver::ops::store_lsb0::<u32, ::device_driver::ops::LE>(
                    raw,
                    0,
                    24,
                    &mut self.bits,
                )
            };
        }
    }

    impl From<[u8; 3]> for FooFieldsIn {
        fn from(bits: [u8; 3]) -> Self {
            Self { bits }
        }
    }

    impl From<FooFieldsIn> for [u8; 3] {
        fn from(val: FooFieldsIn) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for FooFieldsIn {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("FooFieldsIn");

            d.field("value", &self.value());

            d.finish()
        }
    }

    impl core::ops::BitAnd for FooFieldsIn {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for FooFieldsIn {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for FooFieldsIn {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for FooFieldsIn {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for FooFieldsIn {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for FooFieldsIn {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for FooFieldsIn {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    /// Enum containing all possible field set types
    pub enum FieldSetValue {
        FooFieldsIn(FooFieldsIn),
    }
    impl core::fmt::Debug for FieldSetValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::FooFieldsIn(val) => core::fmt::Debug::fmt(val, f),

                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }

    impl From<FooFieldsIn> for FieldSetValue {
        fn from(val: FooFieldsIn) -> Self {
            Self::FooFieldsIn(val)
        }
    }
}
