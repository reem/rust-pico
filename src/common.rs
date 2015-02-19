use std::{mem, raw};

pub unsafe fn slice_to_mut_pair<'a, T>(slice: &'a mut &[T]) -> (&'a mut *const T, &'a mut usize) {
    let raw = mem::transmute::<_, &mut raw::Slice<T>>(slice);
    (&mut raw.data, &mut raw.len)
}

