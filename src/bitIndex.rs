#![allow(dead_code)]

pub const PAGE_ALIGN: usize = 64;
pub const READ_SIZE: usize = 8;
pub const PAGE_SIZE: usize = (PAGE_ALIGN-(READ_SIZE-1)) as usize;


#[repr(C)]
#[repr(align(64))]
pub struct PackedPage {
  // We add bytes to prevent a bad read off the end
  pub ints: [u8; PAGE_SIZE + (READ_SIZE-1)],
}

impl PackedPage {
  // Mask is always (1<<size) -1
  pub fn get(&self, index: u32, size: u32, mask: u64) -> u32 {
    let bit_offset = index * size;
    let byte_offset = bit_offset >> 3;
    let shift = bit_offset & 7;
    // unaligned read from byte offset
    let dword : u64 = unsafe {*( (& self.ints[byte_offset as usize]) as *const u8 as *const u64)};
    ((dword >> shift) & mask) as u32
  }

  // Mask is always (1<<size) -1
  pub fn put(&mut self, index: u32, value: u32, size: u32, mask: u64) -> () {
    let bit_offset = index * size;
    let byte_offset = bit_offset >> 3;
    let shift = bit_offset & 7;
    let ptr = (& self.ints[byte_offset as usize]) as *const u8 as *mut u64;
    let dword : u64 = unsafe { *ptr };
    let updated = dword & !(mask << shift) | ((value as u64) << shift);
    unsafe { * ptr = updated }
  }
}

#[cfg(test)]
mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  fn put() {
    let mut pp = PackedPage {
      ints: [0; PAGE_SIZE + (READ_SIZE-1)],
    };
    let size = 11;
    let max_value = 1 << (size + 1) - 1;
    let mask = (1u64 << size) - 1u64;
    let entries = (PAGE_SIZE * 8) / 11;
    let multiplier = max_value/entries;


    for i in 0..entries {
      println!("{}/{} = {}", i, entries - 1, i * multiplier);
      pp.put(i as u32, (i * multiplier) as u32, size, mask);
    }
    for i in 0..entries {
      println!("{} : {}", i, pp.get(i as u32, size, mask));
      assert!(pp.get(i as u32, size, mask) == (i * multiplier) as u32);
    }
  }

  // extern crate test;
  // use test::Bencher;

  // #[bench]
  // fn get_bench(b: &mut Bencher) {

  // }

}
