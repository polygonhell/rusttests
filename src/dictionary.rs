// Dictionary that works over a set of pages
#![allow(unused_variables)]
#![allow(dead_code)]


use crate::paged_vector::{PagedVector};

// Could be more compact!
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct ArrayPosition {
  pos : u64,
  len : u32
}

struct ArrayDictionary<'a, T> {
  refs: PagedVector<'a, ArrayPosition>,
  arr: PagedVector<'a, T>,
  entries: u32,
}


impl<'a, T> ArrayDictionary<'a, T> {
  fn append(&self, vs: &[T]) -> u32 {
    0
  }

}