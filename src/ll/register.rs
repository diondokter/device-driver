use core::fmt::{Debug, Formatter, UpperHex};

/// Error type for type conversion errors
pub struct ConversionError<T: UpperHex + Debug> {
    /// The raw value that was tried to have converted
    pub raw: T,
}

impl<T: UpperHex + Debug> Debug for ConversionError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConversionError")
            .field("raw", &format_args!("0x{:X}", self.raw))
            .finish()
    }
}

/// Trait for reading and writing registers
pub trait RegisterInterface {
    /// The type representation of the address
    type Address;
    /// The type representation of the errors the interface can give
    type InterfaceError: Debug;

    // To consider: Right now we're using byte arrays for interfacing with registers.
    // This could also be [`bitarray`](https://crates.io/crates/bitarray).
    // Pro: Better support for i.e. 7-bit registers.
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

/// Trait for modifying single bits from register
pub trait BitModifyRegisterInterface: RegisterInterface {
    /// Writes register with changed bits mask
    fn bit_modify_register(
        &mut self,
        address: Self::Address,
        mask: &[u8],
        value: &[u8],
    ) -> Result<(), Self::InterfaceError>;
}

/// Defines a register interface for a low level device.
///
/// Format:
///
/// - `AccessSpecifier` = `WO` (write-only) | `RO` (read-only) | `RW` (read-write)
/// - `FieldType` = any int type
/// - `SomeType` = any type that implements Into<FieldType> the field can be written and TryFrom<FieldType> if the field can be read
/// - `RegisterBitOrder` = optional (can be left out. default = LSB) or `LSB` (Least Significant Bit) | `MSB` (Most Significant Bit) =>
/// This follows uses the ordering semantics of [bitvec::slice::BitSlice] when used with [bitvec::field::BitField].
/// - `FieldBitOrder` = optional (can be left out. default = BE) or `:LE` (Little Endian) | `:BE` (Big Endian) | `:NE` (Native Endian) =>
/// Specifies how the bits are read. Native Endian specifies that the CPU Architecture decides if it's LE or BE.
/// This only makes sense to specify for int types that have more than 1 byte.
///
/// ```ignore
/// implement_registers!(
///     /// Possible docs for register set
///     DeviceName.register_set_name<RegisterAddressType> = {
///         /// Possible docs for register
///         register_name(AccessSpecifier, register_address, register_size) = {
///             /// Possible docs for register field
///             field_name: FieldType = AccessSpecifier inclusive_start_bit_index..exclusive_end_bit_index,
///             /// This field has a conversion and uses an inclusive range
///             field_name: FieldType as SomeType = AccessSpecifier inclusive_start_bit_index..=inclusive_end_bit_index,
///             /// This field is read with a specified endianness
///             field_name: FieldType:FieldBitOrder = AccessSpecifier inclusive_start_bit_index..exclusive_end_bit_index,
///         },
///         /// This register is present at multiple addresses and has a specified register bit order
///         register_name(AccessSpecifier, [register_address, register_address, register_address], register_size) = RegisterBitOrder {
///
///         },
///     }
/// );
/// ```
///
/// See the examples folder for actual examples
///
#[macro_export]
macro_rules! implement_registers {
    (
        $(#[$register_set_doc:meta])*
        $device_name:ident.$register_set_name:ident<$register_address_type:ty> = {
            $(
                $(#[doc=$register_doc:literal])*
                $(#[generate($($generate_list:tt)*)])?
                $register_name:ident($register_access_specifier:tt, $register_address:tt, $register_size:expr) = $($register_bit_order:ident)? {
                    $(
                        $(#[$field_doc:meta])*
                        $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        $(#[$register_set_doc])*
        pub mod $register_set_name {
            use super::*;
            use device_driver::ll::register::{RegisterInterface, ConversionError};
            use device_driver::ll::register::BitModifyRegisterInterface;
            use device_driver::ll::LowLevelDevice;
            use device_driver::_implement_register;
            use device_driver::_implement_register_field;
            use device_driver::_get_bit_order;
            use device_driver::_load_with_endianness;
            use device_driver::_store_with_endianness;
            use device_driver::generate_if_debug_keyword;

            impl<'a, I: HardwareInterface> $device_name<I>
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
                    $(#[doc = $register_doc])*
                    pub fn $register_name(&'a mut self) -> RegAccessor<'a, I, $register_name::R, $register_name::W> {
                        RegAccessor::new(&mut self.interface)
                    }
                )*
            }

            $(
                $(#[doc = $register_doc])*
                pub mod $register_name {
                    use super::*;

                    _implement_register!(
                        #[generate($($($generate_list)*)*)]
                        ($register_name, $register_access_specifier, $register_address, $register_size, $register_address_type, _get_bit_order!($($register_bit_order)*)) {
                            $(
                                $(#[$field_doc])*
                                $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                            ),*
                        }
                    );
                }
            )*
        }
    };
}

/// Internal macro. Do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! _implement_register {
    // This arm implements the array read part (but not read-only)
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, @R, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Reader struct for the register
        #[derive(Copy, Clone)]
        pub struct R([u8; $register_size]);

        impl R {
            /// Create a zeroed reader
            pub const fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Creates a reader with the given value.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn from_raw(value: [u8; $register_size]) -> Self {
                Self(value)
            }

            /// Gets the raw value of the writer.
            pub const fn get_raw(&self) -> [u8; $register_size] {
                self.0
            }

            $(
                _implement_register_field!(@R, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        generate_if_debug_keyword!(
            impl core::fmt::Debug for R {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(concat!(stringify!($register_name), "::R"))
                        .field("raw", &device_driver::utils::SliceHexFormatter::new(&self.0))
                        $(
                            .field(stringify!($field_name), &self.$field_name())
                        )*
                        .finish()
                }
            },
            impl core::fmt::Debug for R {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(concat!(stringify!($register_name), "::R"))
                        .field("raw", &device_driver::utils::SliceHexFormatter::new(&self.0))
                        .finish()
                }
            },
            $($generate_list)*
        );

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
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, @R, $register_address:expr, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Reader struct for the register
        #[derive(Copy, Clone)]
        pub struct R(pub [u8; $register_size]);

        impl R {
            /// Create a zeroed reader
            pub const fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Creates a reader with the given value.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn from_raw(value: [u8; $register_size]) -> Self {
                Self(value)
            }

            /// Gets the raw value of the writer.
            pub const fn get_raw(&self) -> [u8; $register_size] {
                self.0
            }

            $(
                _implement_register_field!(@R, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        generate_if_debug_keyword!(
            impl core::fmt::Debug for R {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(concat!(stringify!($register_name), "::R"))
                        .field("raw", &device_driver::utils::SliceHexFormatter::new(&self.0))
                        $(
                            .field(stringify!($field_name), &self.$field_name())
                        )*
                        .finish()
                }
            },
            impl core::fmt::Debug for R {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(concat!(stringify!($register_name), "::R"))
                        .field("raw", &device_driver::utils::SliceHexFormatter::new(&self.0))
                        .finish()
                }
            },
            $($generate_list)*
        );

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
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, @W, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Writer struct for the register
        #[derive(Debug, Copy, Clone)]
        pub struct W([u8; $register_size]);

        impl W {
            /// Create a zeroed writer
            pub const fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Creates a writer with the given value.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn from_raw(value: [u8; $register_size]) -> Self {
                Self(value)
            }

            /// Sets the raw value of the writer.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn set_raw(self, value: [u8; $register_size]) -> Self {
                Self(value)
            }

            $(
                _implement_register_field!(@W, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Writes the value returned by the closure to the register
            pub fn write_index<F>(&mut self, index: usize, f: F) -> Result<(), I::InterfaceError>
            where
                for<'w> F: FnOnce(&'w mut W) -> &'w mut W,
            {
                let mut w = W::zero();
                let _ = f(&mut w);
                self.write_index_direct(index, w)
            }

            /// Writes the w value to the register
            fn write_index_direct(&mut self, index: usize, w: W) -> Result<(), I::InterfaceError> {
                let addresses = [$($register_address,)*];
                self.interface.write_register(addresses[index], &w.0)?;
                Ok(())
            }
        }
    };
    // This arm implements the single write part (but not write-only)
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, @W, $register_address:expr, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        /// Writer struct for the register
        #[derive(Debug, Copy, Clone)]
        pub struct W([u8; $register_size]);

        impl W {
            /// Create a zeroed writer
            pub const fn zero() -> Self {
                Self([0; $register_size])
            }

            /// Creates a writer with the given value.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn from_raw(value: [u8; $register_size]) -> Self {
                Self(value)
            }

            /// Sets the raw value of the writer.
            ///
            /// Be careful because you may inadvertently set invalid values
            pub const fn set_raw(self, value: [u8; $register_size]) -> Self {
                Self(value)
            }

            $(
                _implement_register_field!(@W, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
            )*
        }

        impl<'a, I> RegAccessor<'a, I, R, W>
        where
            I: RegisterInterface<Address = $register_address_type>,
        {
            /// Writes the value returned by the closure to the register
            pub fn write<F>(&mut self, f: F) -> Result<(), I::InterfaceError>
            where
                for<'w> F: FnOnce(&'w mut W) -> &'w mut W,
            {
                let mut w = W::zero();
                let _ = f(&mut w);
                self.write_direct(w)
            }

            /// Writes the w value to the register
            fn write_direct(&mut self, w: W) -> Result<(), I::InterfaceError> {
                self.interface.write_register($register_address, &w.0)?;
                Ok(())
            }
        }
    };
    // This arm implements both array read and write parts
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, RW, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @R, [$($register_address,)*], $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @W, [$($register_address,)*], $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
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
                for<'w> F: FnOnce(R, &'w mut W) -> &'w mut W,
            {
                let r = self.read_index(index)?;
                let mut w = W(r.0.clone());

                let _ = f(r, &mut w);
                self.write_index_direct(index, w)?;
                Ok(())
            }
        }
    };
    // This arm implements both single read and write parts
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, RW, $register_address:expr, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @R, $register_address, $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @W, $register_address, $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
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
                for<'w> F: FnOnce(R, &'w mut W) -> &'w mut W,
            {
                let r = self.read()?;
                let mut w = W(r.0.clone());

                let _ = f(r, &mut w);
                self.write_direct(w)?;
                Ok(())
            }
        }
    };

        // This arm implements both single RWB
        (
            #[generate($($generate_list:tt)*)]
            ($register_name:ident, RWB, $register_address:expr, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
                $(
                    $(#[$field_doc:meta])*
                    $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
                ),*
            }
        ) => {
            _implement_register!(
                #[generate($($generate_list)*)]
                ($register_name, RW, $register_address, $register_size, $register_address_type, $register_bit_order) {
                    $(
                        $(#[$field_doc])*
                        $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                    ),*
                }
            );
            _implement_register!(
                #[generate($($generate_list)*)]
                ($register_name, @B, $register_address, $register_size, $register_address_type, $register_bit_order) {
                    $(
                        $(#[$field_doc])*
                        $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                    ),*
                }
            );
        };
        // This arm implements the array write part (but not write-only)
        (
            #[generate($($generate_list:tt)*)]
            ($register_name:ident, @B, [$($register_address:expr),* $(,)?], $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
                $(
                    $(#[$field_doc:meta])*
                    $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
                ),*
            }
        ) => {
            /// Writer struct for the register
            #[derive(Debug, Copy, Clone)]
            pub struct B([u8; $register_size], [u8; $register_size]);

            impl B {
                /// Create a zeroed writer
                pub const fn zero() -> Self {
                    Self([0; $register_size], [0; $register_size])
                }

                /// Creates a writer with the given value.
                ///
                /// Be careful because you may inadvertently set invalid values
                pub const fn from_raw(value: [u8; $register_size]) -> Self {
                    Self(value, [0; $register_size])
                }

                $(
                    _implement_register_field!(@B, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
                )*
            }

            impl<'a, I> RegAccessor<'a, I, R, W>
            where
                I: BitModifyRegisterInterface<Address = $register_address_type>,
            {
                /// Writes the w value to the register
                fn write_modified(&mut self, b: B) -> Result<(), I::InterfaceError> {
                    self.interface.bit_modify_register($register_address, &b.1, &b.0)?;
                    Ok(())
                }

                pub fn bit_modify<F>(&mut self, f: F) -> Result<(), I::InterfaceError>
                where
                    for<'b> F: FnOnce(&'b mut B) -> &'b mut B,
                {
                    let mut b = B([0; $register_size], [0; $register_size]);

                    let _ = f(&mut b);
                    self.write_modified(b)?;
                    Ok(())
                }
            }
        };
        // This arm implements the single write part (but not write-only)
        (
            #[generate($($generate_list:tt)*)]
            ($register_name:ident, @B, $register_address:expr, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
                $(
                    $(#[$field_doc:meta])*
                    $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
                ),*
            }
        ) => {
            /// Writer struct for the register
            #[derive(Debug, Copy, Clone)]
            pub struct B([u8; $register_size], [u8; $register_size]);

            impl B {
                /// Create a zeroed writer
                pub const fn zero() -> Self {
                    Self([0; $register_size], [0; $register_size])
                }

                /// Creates a writer with the given value.
                ///
                /// Be careful because you may inadvertently set invalid values
                pub const fn from_raw(value: [u8; $register_size]) -> Self {
                    Self(value, [0; $register_size])
                }

                $(
                    _implement_register_field!(@B, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range);
                )*
            }

            impl<'a, I> RegAccessor<'a, I, R, W>
            where
                I: BitModifyRegisterInterface<Address = $register_address_type>,
            {
                /// Writes the w value to the register
                fn write_modified(&mut self, b: B) -> Result<(), I::InterfaceError> {
                    self.interface.bit_modify_register($register_address, &b.1, &b.0)?;
                    Ok(())
                }

                pub fn bit_modify<F>(&mut self, f: F) -> Result<(), I::InterfaceError>
                where
                    for<'b> F: FnOnce(&'b mut B) -> &'b mut B,
                {
                    let mut b = B([0; $register_size], [0; $register_size]);

                    let _ = f(&mut b);
                    self.write_modified(b)?;
                    Ok(())
                }
            }
        };



    // This arm implements the read part and disables write
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, RO, $register_address:tt, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @R, $register_address, $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        /// Empty writer. This means this register is read-only
        pub type W = ();
    };
    // This arm implements the write part and disables read
    (
        #[generate($($generate_list:tt)*)]
        ($register_name:ident, WO, $register_address:tt, $register_size:expr, $register_address_type:ty, $register_bit_order:ty) {
            $(
                $(#[$field_doc:meta])*
                $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = $field_access_specifier:tt $field_bit_range:expr
            ),*
        }
    ) => {
        _implement_register!(
            #[generate($($generate_list)*)]
            ($register_name, @W, $register_address, $register_size, $register_address_type, $register_bit_order) {
                $(
                    $(#[$field_doc])*
                    $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = $field_access_specifier $field_bit_range
                ),*
            }
        );

        /// Empty reader. This means this register is write-only
        pub type R = ();
    };
}

/// Internal macro. Do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! _implement_register_field {
    // Read without 'as' convert
    (@R, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? = RO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&self) -> $field_type {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBits;

            _load_with_endianness!(self.0.as_bits::<$register_bit_order>()[$field_bit_range], $($field_bit_order)?)
        }
    };
    // Read with 'as' convert
    (@R, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? as $field_convert_type:ty = RO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&self) -> Result<$field_convert_type, ConversionError<$field_type>> {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBits;
            use core::convert::TryInto;

            let raw: $field_type = _load_with_endianness!(self.0.as_bits::<$register_bit_order>()[$field_bit_range], $($field_bit_order)?);
            raw.try_into().map_err(|_| ConversionError { raw })
        }
    };
    (@R, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = WO $field_bit_range:expr) => {
        // Empty on purpose
    };
    (@R, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = RW $field_bit_range:expr) => {
        _implement_register_field!(@R, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = RO $field_bit_range);
        _implement_register_field!(@R, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = WO $field_bit_range);
    };
    (@W, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = RO $field_bit_range:expr) => {
        // Empty on purpose
    };
    // Write without 'as' convert
    (@W, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&mut self, value: $field_type) -> &mut Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            _store_with_endianness!(self.0.as_bits_mut::<$register_bit_order>()[$field_bit_range], value, $($field_bit_order)?);

            self
        }
    };
    // Write with 'as' convert
    (@W, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? as $field_convert_type:ty = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&mut self, value: $field_convert_type) -> &mut Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            let raw_value: $field_type = value.into();
            _store_with_endianness!(self.0.as_bits_mut::<$register_bit_order>()[$field_bit_range], raw_value, $($field_bit_order)?);

            self
        }
    };
    (@W, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = RW $field_bit_range:expr) => {
        _implement_register_field!(@W, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = RO $field_bit_range);
        _implement_register_field!(@W, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = WO $field_bit_range);
    };


    // Write without 'as' convert
    (@B, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&mut self, value: $field_type) -> &mut Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            _store_with_endianness!(self.0.as_bits_mut::<$register_bit_order>()[$field_bit_range], value, $($field_bit_order)?);
            _store_with_endianness!(self.1.as_bits_mut::<$register_bit_order>()[$field_bit_range], 0xffffffffffffffff as $field_type, $($field_bit_order)?);

            self
        }
    };
    // Write with 'as' convert
    (@B, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? as $field_convert_type:ty = WO $field_bit_range:expr) => {
        $(#[$field_doc])*
        pub fn $field_name(&mut self, value: $field_convert_type) -> &mut Self {
            use device_driver::bitvec::prelude::*;
            use device_driver::bitvec::view::AsBitsMut;

            let raw_value: $field_type = value.into();
            _store_with_endianness!(self.0.as_bits_mut::<$register_bit_order>()[$field_bit_range], raw_value, $($field_bit_order)?);
            _store_with_endianness!(self.1.as_bits_mut::<$register_bit_order>()[$field_bit_range], 0xffffffffffffffff as $field_type, $($field_bit_order)?);


            self
        }
    };
    (@B, $register_bit_order:ty, $(#[$field_doc:meta])* $field_name:ident: $field_type:ty $(:$field_bit_order:ident)? $(as $field_convert_type:ty)? = RW $field_bit_range:expr) => {
        _implement_register_field!(@B, $register_bit_order, $(#[$field_doc])* $field_name: $field_type $(:$field_bit_order)? $(as $field_convert_type)? = WO $field_bit_range);
    };
}

/// Internal macro. Do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! _get_bit_order {
    () => {
        Lsb0
    };
    (LSB) => {
        Lsb0
    };
    (MSB) => {
        Msb0
    };
}

/// Internal macro. Do not use.
///
/// Load the value from the [bitvec::field::BitField] with the given endianness
#[macro_export]
#[doc(hidden)]
macro_rules! _load_with_endianness {
    // Load the value from the field with the default ordering
    ($field:expr, ) => {
        _load_with_endianness!($field, BE)
    };
    // Load the value from the field with the Big Endian ordering
    ($field:expr, BE) => {
        $field.load_be()
    };
    // Load the value from the field with the Little Endian ordering
    ($field:expr, LE) => {
        $field.load_le()
    };
    // Load the value from the field with the Native Endian ordering
    ($field:expr, NE) => {
        $field.load()
    };
}

/// Internal macro. Do not use.
///
/// Store the value into the [bitvec::field::BitField] with the given endianness
#[macro_export]
#[doc(hidden)]
macro_rules! _store_with_endianness {
    // Store the value into the field with the default ordering
    ($field:expr, $value:expr, ) => {
        _store_with_endianness!($field, $value, BE)
    };
    // Store the value into the field with the Big Endian ordering
    ($field:expr, $value:expr, BE) => {
        $field.store_be($value)
    };
    // Store the value into the field with the Little Endian ordering
    ($field:expr, $value:expr, LE) => {
        $field.store_le($value)
    };
    // Store the value into the field with the Native Endian ordering
    ($field:expr, $value:expr, NE) => {
        $field.store($value)
    };
}

/// Internal macro. Do not use.
///
/// A TT muncher that will place the `true` parameter if the list contains the `Debug` keyword
/// and the `false` parameter if it does not.
#[macro_export]
#[doc(hidden)]
macro_rules! generate_if_debug_keyword {
    // There's no Debug keyword
    ($true:item, $false:item, ) => {
        $false
    };
    // There's only a Debug keyword
    ($true:item, $false:item, Debug) => {
        $true
    };
    // There's a Debug keyword
    ($true:item, $false:item, Debug, $($list:tt)*) => {
        generate_if_debug_keyword!($true, $false, Debug);
    };
    // There's only a different keyword
    ($true:item, $false:item, $keyword:ident) => {
        generate_if_debug_keyword!($true, $false, );
    };
    // There's a different keyword, so we need to continue munching
    ($true:item, $false:item, $keyword:ident, $($list:tt)*) => {
        generate_if_debug_keyword!($true, $false, $($list)*);
    };
}
