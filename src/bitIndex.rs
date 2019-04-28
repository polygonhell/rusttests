#![allow(dead_code)]

const PAGE_ALIGN: u32 = 64;
pub const PAGE_SIZE: usize = 64;

#[repr(C)]
#[repr(align(64))]
pub struct PackedPage {
  pub ints: [u32; PAGE_SIZE / 4 + 1],
}

impl PackedPage {
  // Mask is always (1<<size) -1
  pub fn get(&self, index: u32, size: u32, mask: u64) -> u32 {
    let bit_offset = index * size;
    let int_offset = (bit_offset >> 5) as usize;
    let shift = 64 - ((bit_offset & 31) + size);
    let temp: u64 = ((self.ints[int_offset] as u64) << 32u64) + (self.ints[int_offset + 1] as u64);
    ((temp >> shift) & mask) as u32
  }

  // Mask is always (1<<size) -1
  fn put(&mut self, index: u32, value: u32, size: u32, mask: u64) -> () {
    let bit_offset = index * size;
    let int_offset = (bit_offset >> 5) as usize;
    let temp: u64 = ((self.ints[int_offset] as u64) << 32u64) + (self.ints[int_offset + 1] as u64);
    let shift = 64 - ((bit_offset & 31) + size);
    let mask = mask << shift;
    let new_val = (temp & (!mask)) | ((value as u64) << shift);
    self.ints[int_offset] = (new_val >> 32) as u32;
    self.ints[int_offset + 1] = new_val as u32;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn put() {
    let mut pp = PackedPage {
      ints: [0; PAGE_SIZE / 4 + 1],
    };
    let size = 11;
    let mask = (1u64 << size) - 1u64;
    let max = (PAGE_SIZE * 8) / 11;


    for i in 0..max {
      println!("{}/{}", i, max);
      pp.put(i as u32, (i + 5) as u32, size, mask);
    }
    for i in 0..max {
      println!("{} : {}", i, pp.get(i as u32, size, mask));
      assert!(pp.get(i as u32, size, mask) == (i + 5) as u32);
    }
  }

  // extern crate test;
  // use test::Bencher;

  // #[bench]
  // fn get_bench(b: &mut Bencher) {

  // }

}
