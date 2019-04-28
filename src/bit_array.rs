
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};



struct CacheAlignedMem {
  ptr : *mut u8,
}

impl CacheAlignedMem {
  fn new(size: u32) -> CacheAlignedMem {
    CacheAlignedMem {
      ptr : unsafe { System.alloc_zeroed(Layout::from_size_align(size as usize, 64).unwrap()) },
    }
  }
}


// Align it to a cache line
pub struct BitArray {
  bytes: Vec<u8>,
  mask: u64,
  word_size: u32,
}

impl BitArray {
  pub fn new(size: u32, word_size: u32) -> BitArray {
    // 7 additional bytes to cope with the 64 bit read in get/put
    let num_bytes = (((size * word_size + 7) >> 3) + 7) as usize;
    BitArray {
      bytes: vec![0; num_bytes],
      word_size: word_size,
      mask: (1u64 << word_size) - 1,
    }
  }

  pub fn get(&self, index: u32) -> u32 {
    let bit_offset = index * self.word_size;
    let byte_offset = bit_offset >> 3;
    let shift = bit_offset & 7;
    // unaligned read from byte offset
    let ptr = unsafe { self.bytes.as_ptr().offset(byte_offset as isize) as *const u64 };
    let dword: u64 = unsafe { *ptr };
    ((dword >> shift) & self.mask) as u32
  }

  pub fn put(&mut self, index: u32, value: u32) -> () {
    let bit_offset = index * self.word_size;
    let byte_offset = bit_offset >> 3;
    let shift = bit_offset & 7;
    let ptr = unsafe { self.bytes.as_ptr().offset(byte_offset as isize) as *mut u64 };
    let dword: u64 = unsafe { *ptr };
    let updated = dword & !(self.mask << shift) | ((value as u64) << shift);
    unsafe { *ptr = updated }
  }
}

#[cfg(test)]
pub mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  pub fn foo() {
    let size = 11;
    let entries = 40;
    let mut pp = BitArray::new(entries, size);
    let max_value = 1 << (size + 1) - 1;
    let multiplier = max_value / entries;

    println!("Here");

    for i in 0..entries {
      println!("{}/{} = {}", i, entries - 1, i * multiplier);
      pp.put(i as u32, (i * multiplier) as u32);
    }
    for i in 0..entries {
      println!("{} : {}", i, pp.get(i as u32));
      assert!(pp.get(i as u32) == (i * multiplier) as u32);
    }
  }
}
