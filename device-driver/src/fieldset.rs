use crate::FieldsetMetadata;

/// # Safety:
/// Implers of this trait will get their memory changed through a byte slice.
/// So make sure the type has a stable ABI and has no padding bits.
pub unsafe trait Fieldset {
    /// Metadata describing some properties of the fieldset
    const METADATA: FieldsetMetadata;
    /// Get a zero-initialized instance of the fieldset
    const ZERO: Self;

    /// Access the inner buffer as a slice
    fn get_inner_buffer(&self) -> &[u8];
    /// Access the inner buffer as a mutable slice
    fn get_inner_buffer_mut(&mut self) -> &mut [u8];
}

unsafe impl<T: Fieldset, const N: usize> Fieldset for [T; N] {
    const METADATA: FieldsetMetadata = T::METADATA;
    const ZERO: Self = [T::ZERO; N];

    fn get_inner_buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.as_ptr().cast::<u8>(), core::mem::size_of::<Self>())
        }
    }

    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.as_mut_ptr().cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}
