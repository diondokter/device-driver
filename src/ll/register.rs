use core::fmt::Debug;

/// Error type for type conversion errors
#[derive(Debug)]
pub struct ConversionError;

/// Trait for reading and writing registers
pub trait RegisterInterface {
    /// The type representation of the address
    type Address;
    /// The type representation of the errors the interface can give
    type InterfaceError: Debug;

    // To consider: Right now we're using byte arrays for interfacing with registers.
    // This could also be bitarray. Pro: Better support for i.e. 7-bit registers.
    // Con: More elaborate to work with in most cases.

    /// Reads the register at the given address and puts the data in the value parameter
    fn read_register(
        &mut self,
        address: Self::Address,
        value: &mut [u8],
    ) -> Result<(), Self::InterfaceError>;

    /// Writes the value to the register at the given address
    fn write_register(
        &mut self,
        address: Self::Address,
        value: &[u8],
    ) -> Result<(), Self::InterfaceError>;
}

#[macro_export]
macro_rules! implement_registers {
    (
        $(#[$register_set_doc:meta])*
        $device_name:ident.$register_set_name:ident<$register_address_type:ty> = {
            $(
                $(#[$register_doc:meta])*
                $register_name:ident($register_access_specifier:tt, $register_address:tt, $register_size:expr) = {
                    $(
                        $(#[$field_doc:meta])*
                        $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        $(#[$register_set_doc])*
        pub mod $register_set_name {
            use super::*;
            use device_driver::ll::register::{RegisterInterface, ConversionError};
            use device_driver::ll::LowLevelDevice;
            use device_driver::implement_register;
            use device_driver::implement_register_field;

            impl<'a, I> $device_name<I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                $(#[$register_set_doc])*
                pub fn $register_set_name(&'a mut self) -> RegisterSet<'a, I> {
                    RegisterSet::new(&mut self.interface)
                }
            }

            /// A struct that borrows the interface from the device.
            /// It implements the read and/or write functionality for the registers.
            pub struct RegAccessor<'a, I, R, W>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                interface: &'a mut I,
                phantom: core::marker::PhantomData<(R, W)>,
            }

            impl<'a, I, R, W> RegAccessor<'a, I, R, W>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                fn new(interface: &'a mut I) -> Self {
                    Self {
                        interface,
                        phantom: Default::default(),
                    }
                }
            }

            /// A struct containing all the register definitions
            pub struct RegisterSet<'a, I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                interface: &'a mut I,
            }

            impl<'a, I> RegisterSet<'a, I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                fn new(interface: &'a mut I) -> Self {
                    Self { interface }
                }

                $(
                    $(#[$register_doc])*
                    pub fn $register_name(&'a mut self) -> RegAccessor<'a, I, $register_name::R, $register_name::W> {
                        RegAccessor::new(&mut self.interface)
                    }
                )*
            }

            $(
                $(#[$register_doc])*
                pub mod $register_name {
                    use super::*;

                    implement_register!(
                        ($register_access_specifier, $register_address, $register_size, $register_address_type) {
                            $(
                                $(#[$field_doc])*
                                $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                            ),*
                        }
                    );
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! implement_register {
    // This arm implements the array read part (but not read-only)
    (
        $(#[$register_doc:meta])*
        (@R, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Reader struct for the register
        pub struct R([u8; $register_size]);

        impl R {
            fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Gets the raw value of the writer.
            pub fn get_raw(&self) -> [u8; $register_size] {
                self.0
            }

            $(
                implement_register_field!(@R, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Reads the register
            pub fn read_index(&mut self, index: usize) -> Result<R, I::InterfaceError> {
                let mut r = R::zero();
                let addresses = [$($register_address,)*];
                self.interface.read_register(addresses[index], &mut r.0)?;
                Ok(r)
            }
        }
    };
    // This arm implements the single read part (but not read-only)
    (
        $(#[$register_doc:meta])*
        (@R, $register_address:expr, $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Reader struct for the register
        pub struct R([u8; $register_size]);

        impl R {
            fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Gets the raw value of the writer.
            pub fn get_raw(&self) -> [u8; $register_size] {
                self.0
            }

            $(
                implement_register_field!(@R, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Reads the register
            pub fn read(&mut self) -> Result<R, I::InterfaceError> {
                let mut r = R::zero();
                self.interface.read_register($register_address, &mut r.0)?;
                Ok(r)
            }
        }
    };
    // This arm implements the array write part (but not write-only)
    (
        $(#[$register_doc:meta])*
        (@W, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Writer struct for the register
        pub struct W([u8; $register_size]);

        impl W {
            fn zero() -> Self {
                Self([0; $register_size])
            }
            
            /// Sets the raw value of the writer.
            ///
            /// Be careful because you may inadvertendly set invalid values
            pub fn set_raw(&mut self, value: [u8; $register_size]) {
                self.0 = value;
            }

            $(
                implement_register_field!(@W, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Writes the value returned by the closure to the register
            pub fn write_index<F>(&mut self, index: usize, f: F) -> Result<(), I::InterfaceError>
            where
                F: FnOnce(W) -> W,
            {
                let w = f(W::zero());
                let addresses = [$($register_address,)*];
                self.interface.write_register(addresses[index], &w.0)?;
                Ok(())
            }
        }
    };
    // This arm implements the single write part (but not write-only)
    (
        $(#[$register_doc:meta])*
        (@W, $register_address:expr, $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Writer struct for the register
        pub struct W([u8; $register_size]);

        impl W {
            fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Sets the raw value of the writer.
            ///
            /// Be careful because you may inadvertendly set invalid values
            pub fn set_raw(&mut self, value: [u8; $register_size]) {
                self.0 = value;
            }

            $(
                implement_register_field!(@W, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Writes the value returned by the closure to the register
            pub fn write<F>(&mut self, f: F) -> Result<(), I::InterfaceError>
            where
                F: FnOnce(W) -> W,
            {
                let w = f(W::zero());
                self.interface.write_register($register_address, &w.0)?;
                Ok(())
            }
        }
    };
    // This arm implements both array read and write parts
    (
        $(#[$register_doc:meta])*
        (RW, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        implement_register!(
            $(#[$register_doc:meta])*
            (@R, [$($register_address,)*], $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );
        implement_register!(
            $(#[$register_doc:meta])*
            (@W, [$($register_address,)*], $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Reads the register, gives the value to the closure and writes back the value returned by the closure
            pub fn modify_index<F>(&mut self, index: usize, f: F) -> Result<(), I::InterfaceError>
            where
                F: FnOnce(R, W) -> W,
            {
                let r = self.read_index(index)?;
                let w = W(r.0.clone());

                let w = f(r, w);

                self.write_index(index, |_| w)?;
                Ok(())
            }
        }
    };
    // This arm implements both single read and write parts
    (
        $(#[$register_doc:meta])*
        (RW, $register_address:expr, $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        implement_register!(
            $(#[$register_doc:meta])*
            (@R, $register_address, $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );
        implement_register!(
            $(#[$register_doc:meta])*
            (@W, $register_address, $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Reads the register, gives the value to the closure and writes back the value returned by the closure
            pub fn modify<F>(&mut self, f: F) -> Result<(), I::InterfaceError>
            where
                F: FnOnce(R, W) -> W,
            {
                let r = self.read()?;
                let w = W(r.0.clone());

                let w = f(r, w);

                self.write(|_| w)?;
                Ok(())
            }
        }
    };
    // This arm implements the read part and disables write
    (
        $(#[$register_doc:meta])*
        (RO, $register_address:tt, $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        implement_register!(
            $(#[$register_doc:meta])*
            (@R, $register_address, $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        /// Empty writer. This means this register is read-only
        pub type W = ();
    };
    // This arm implements the write part and disables read
    (
        $(#[$register_doc:meta])*
        (WO, $register_address:tt, $register_size:expr, $register_address_type:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        implement_register!(
            $(#[$register_doc:meta])*
            (@W, $register_address, $register_size, $register_address_type) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        /// Empty reader. This means this register is write-only
        pub type R = ();
    };
}

#[macro_export]
macro_rules! implement_register_field {
    // Read without 'as' convert
    (@R, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty = RO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&self) -> $field_type {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBits;

            self.0.as_bits::<Lsb0>()[$field_bit_range].load_be()
        }
    };
    // Read with 'as' convert
    (@R, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty as $field_convert_type:ty = RO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&self) -> Result<$field_convert_type, ConversionError> {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBits;
            use core::convert::TryInto;

            let raw: $field_type = self.0.as_bits::<Lsb0>()[$field_bit_range].load_be();
            raw.try_into().map_err(|_| ConversionError)
        }
    };
    (@R, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = WO $field_bit_range:expr) => {
        // Empty on purpose
    };
    (@R, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = RW $field_bit_range:expr) => {
        implement_register_field!(@R, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = RO $field_bit_range);
        implement_register_field!(@R, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = WO $field_bit_range);
    };
    (@W, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = RO $field_bit_range:expr) => {
        // Empty on purpose
    };
    // Write without 'as' convert
    (@W, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(mut self, value: $field_type) -> Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            self.0.as_bits_mut::<Lsb0>()[$field_bit_range].store_be(value);

            self
        }
    };
    // Write with 'as' convert
    (@W, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty as $field_convert_type:ty = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(mut self, value: $field_convert_type) -> Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            let raw_value: $field_type = value.into();
            self.0.as_bits_mut::<Lsb0>()[$field_bit_range].store_be(raw_value);

            self
        }
    };
    (@W, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(as $field_convert_type:ty)? = RW $field_bit_range:expr) => {
        implement_register_field!(@W, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = RO $field_bit_range);
        implement_register_field!(@W, $(#[$field_doc])* $field_name: $field_type $(as $field_convert_type)? = WO $field_bit_range);
    };
}
