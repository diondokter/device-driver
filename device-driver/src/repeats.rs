use core::marker::PhantomData;

use crate::Address;

#[diagnostic::on_unimplemented(
    label = "this object does not repeat. Use the function variant without `_at` in the name to interact with the object"
)]
#[doc(hidden)]
pub trait Repeating {
    type Index: Clone;

    /// Calculate an address with the index
    #[allow(private_bounds)]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType;
}

#[diagnostic::on_unimplemented(
    label = "this object has a repeat and you must specify an index. Use the function variant with `_at` in the name to interact with the object"
)]
#[doc(hidden)]
pub trait NotRepeating {}
impl NotRepeating for () {}

#[diagnostic::on_unimplemented(
    label = "this object has a repeat, but can't be used with array operations. Avoid using functions with `_array` in the name to interact with the object",
    note = "repeats that use an enum cannot be used as an array"
)]
#[doc(hidden)]
pub trait ArrayRepeating: Repeating {
    const COUNT: usize;
    const STRIDE: i32;

    fn assert_len_and_index(len: usize, index: Self::Index);
}

#[doc(hidden)]
pub struct ArrayRepeat<const COUNT: usize, const STRIDE: i32>;
impl<const COUNT: usize, const STRIDE: i32> Repeating for ArrayRepeat<COUNT, STRIDE> {
    type Index = usize;

    #[track_caller]
    #[inline]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        assert!(
            index < COUNT,
            "Index out of range: {index} (array len: {COUNT})"
        );
        let offset = index as i32 * STRIDE;
        start.add(offset)
    }
}
impl<const COUNT: usize, const STRIDE: i32> ArrayRepeating for ArrayRepeat<COUNT, STRIDE> {
    const COUNT: usize = COUNT;
    const STRIDE: i32 = STRIDE;

    #[track_caller]
    #[inline]
    fn assert_len_and_index(len: usize, index: Self::Index) {
        assert!(
            index < COUNT,
            "index out of range: {index} (array len: {COUNT})"
        );
        assert!(
            len + index <= COUNT,
            "array too long. Requested {len}, max len remaining at requested index is {}",
            COUNT - index,
        );
    }
}

#[doc(hidden)]
pub struct RangeRepeat<const END: usize, const START: usize, const STRIDE: i32>;
impl<const END: usize, const START: usize, const STRIDE: i32> Repeating
    for RangeRepeat<END, START, STRIDE>
{
    type Index = usize;

    #[track_caller]
    #[inline]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        assert!(
            index <= (END - START),
            "Index out of range: {index} (array len: {})",
            (END - START)
        );
        let offset = (START as i32 + index as i32) * STRIDE;
        start.add(offset)
    }
}
impl<const END: usize, const START: usize, const STRIDE: i32> ArrayRepeating
    for RangeRepeat<END, START, STRIDE>
{
    const COUNT: usize = (END - START);
    const STRIDE: i32 = STRIDE;

    #[track_caller]
    #[inline]
    fn assert_len_and_index(len: usize, index: Self::Index) {
        assert!(
            index < Self::COUNT,
            "index out of range: {index} (array len: {})",
            Self::COUNT
        );
        assert!(
            len + index <= Self::COUNT,
            "array too long. Requested {len}, max len remaining at requested index is {}",
            Self::COUNT - index,
        );
    }
}

#[doc(hidden)]
pub trait EnumIndex {
    fn index(&self) -> i32;
}

#[doc(hidden)]
pub struct EnumRepeat<T: Clone + EnumIndex, const STRIDE: i32>(PhantomData<T>);
impl<T: Clone + EnumIndex, const STRIDE: i32> Repeating for EnumRepeat<T, STRIDE> {
    type Index = T;

    #[inline]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        let offset = index.index() * STRIDE;
        start.add(offset)
    }
}
