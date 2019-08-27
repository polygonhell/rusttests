#![allow(unused_variables)]
#![allow(dead_code)]

use std::mem;

const FREE_LIST_VERSION: u8 = 0;
const INITIAL_DB_SIZE: u32 = 256;
const PAGE_SIZE_SHIFT: u8 = 12;
const PAGE_SIZE: usize = 1 << PAGE_SIZE_SHIFT;
const VERSION: Version = Version {
  major_version: 0,
  minor_version: 0,
  patch_level: 0,
  dummy: 0,
};

#[repr(C)]
#[derive(Copy, Clone)]
struct Version {
  // The Verion of the file
  major_version: u16,
  minor_version: u16,
  patch_level: u16,
  dummy: u16,
}

type PageRef = u32;

#[repr(C)]
#[derive(Copy, Clone)]
struct Header {
  version: Version,
  pages: u32, // Current number of pages in file Enough for a 2^46 bytes with 4K pages
  free_list: PageRef, // Ptr to the Free List for allocations
  table_index: PageRef, // Ptr to the Table index
  page_size_shift: u8, // Number of bits to shift to conver a PageRef to an actual address
}

#[repr(C)]
#[derive(Copy, Clone)]
union FreeListBody {
  leaf: FreeListLeaf,
  ptrs: FreeListPtrs,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FreeListLeaf {
  d: [u8; (PAGE_SIZE - 4)],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FreeListPtrs {
  d: [u32; ((PAGE_SIZE - 4) / 4)],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FreeList {
  depth: u8,   // 0 is just a bit map, 1 is ptr's to a bit map, 2 is ptr->ptr to bitmap etc
  version: u8, // Just In case
  padding: u16,
  data: FreeListBody,
}

#[repr(C)]
pub struct Database {
  mmap: MmapMut,
}

#[derive(Debug)]
pub enum DbError {
  Io(std::io::Error),
  Err(String),
}

use std::fs::{OpenOptions};
use std::path::Path;

use memmap::MmapMut;

const INIT_HEADER: Header = Header {
  version: VERSION,
  page_size_shift: PAGE_SIZE_SHIFT,
  free_list: 1,
  table_index: 2,
  pages: INITIAL_DB_SIZE,
};

use crate::paged_vector::{Page};

pub trait PageProvider {
  fn alloc(&mut self, count: usize) -> Vec<u32>;
  fn mut_page(&mut self, i: u32) -> (&mut dyn PageProvider, &mut Page);
  fn page(&self, i: u32) -> &Page;
  fn index_of(&self, page: &Page) -> u32;
}


pub struct MemoryPageProvider {
  pages: Vec<Vec<u8>>
}

impl MemoryPageProvider {
  pub fn new() -> MemoryPageProvider {
    MemoryPageProvider {pages: Vec::new()}
  }
}

impl PageProvider for MemoryPageProvider {
  fn alloc(&mut self, _count: usize) -> Vec<u32> {
    self.pages.push(Vec::with_capacity(PAGE_SIZE));
    vec![(self.pages.len() - 1) as u32]
  }

  fn page(&self, i: u32) -> &Page {
    let page = self.pages[i as usize].as_ptr();
    unsafe { & *(page as *const u8 as *const Page) }
  }

  fn mut_page(&mut self, i: u32) -> (&mut dyn PageProvider, &mut Page) {
    let page = self.pages[i as usize].as_mut_ptr();
    (self, unsafe { &mut *(page as *const u8 as *mut Page) })
  }

  fn index_of(&self, page: &Page) -> u32 {
    let ptr = page as *const Page as *const u8;
    for i in 0..self.pages.len() {
      if (&(self.pages[i])[0] as *const u8) == ptr {
        return i as u32
      }
    }
    panic!("Not Found")
  }

}

impl Database {
  pub fn new(file_name: &str) -> Result<Database, DbError> {
    let path = Path::new(file_name);
    // Fail if the file exists
    if path.exists() {
      return Err(DbError::Err(format!(
        "Failed to create Database: File {} already exists",
        file_name
      )));
    }
    // Create the file
    let file = OpenOptions::new()
      .create_new(true)
      .read(true)
      .write(true)
      .open(path)
      .map_err(DbError::Io)?;
    file
      .set_len((INIT_HEADER.pages as u64) << INIT_HEADER.page_size_shift)
      .map_err(DbError::Io)?;
    let mmap = unsafe { MmapMut::map_mut(&file) }.map_err(DbError::Io)?;

    let db = Database { mmap: mmap };

    let hdr = db.header();
    *hdr = INIT_HEADER;

    // Create the Free list with the first 3 bits set 1 for the header, 1 for the FList itself and 1 for the table table
    let flist = db.free_list();
    flist.init();
    flist.set_arr(&[0, 1, 2]);

    // TODO: initalize the base table
    Ok(db)
  }

  fn header(&self) -> &mut Header {
    unsafe { &mut *(self.mmap.as_ptr() as *mut Header) }
  }

  fn free_list(&self) -> &mut FreeList {
    unsafe {
      let header = self.header();
      &mut *(self
        .mmap
        .as_ptr()
        .offset((header.free_list as isize) << header.page_size_shift)
        as *mut FreeList)
    }
  }
}

impl FreeList {
  fn init(&mut self) -> () {
    assert_eq!(PAGE_SIZE, mem::size_of::<FreeList>());
    self.version = FREE_LIST_VERSION;
    self.depth = 0;
    self.padding = 0;
    self.data.leaf.d = [0; (PAGE_SIZE - 4)];
  }

  fn set(&mut self, index: u32) -> &mut FreeList {
    let depth = self.depth;
    match depth {
      0 => unsafe { self.data.leaf.set(index) },
      _x => println!("Not Implemented : set"),
    }
    self
  }

  fn set_arr(&mut self, index: &[u32]) -> &mut FreeList {
    let depth = self.depth;
    match depth {
      0 => unsafe { self.data.leaf.set_arr(index) },
      _x => println!("Not Implemented : set_arr"),
    }
    self
  }
}

impl FreeListLeaf {
  fn set(&mut self, index: u32) {
    let byte = (index as usize) >> 3;
    let bit = index & 7;
    self.d[byte] = self.d[byte] | (1 << bit);
  }

  fn set_arr(&mut self, is: &[u32]) {
    is.iter().for_each(|x| {
      self.set(*x);
    })
  }

  fn get(&self, index: u32) -> bool {
    let byte = (index as usize) >> 3;
    let bit = index & 7;
    0 != (self.d[byte] & (1 << bit))
  }

  // Find a sequence of consequtive free pages of size
  // TODO: some obvious optimizations for large requests
  fn find_free(&self, size: u32) -> u32 {
    let byte_size = mem::size_of::<FreeListLeaf>();
    for i in 0..byte_size {
      match self.d[i] {
        0xff => (),
        b => {
          // Compute the length of the run at least upto Size
          // get the index of the first unser bit
          let mut index = (i as u32) * 8 + (!b).trailing_zeros();
          let mut run = 1;
          // Just scan the rest of the Bit Array
          for r in (index + 1)..(byte_size as u32 * 8) {
            if run >= size {
              return index;
            };
            if !self.get(r) {
              if run == 0 {
                run = 1;
                index = r;
              } else {
                run = run + 1;
              }
            } else {
              run = 0
            }
          }
        }
      }
    }
    // Zero is never valid
    0
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  pub fn find() {
    let mut p = FreeListLeaf {
      d: [0; (PAGE_SIZE - 4)],
    };
    p.set_arr(&[0, 1, 4, 8, 256]);
    assert!(p.find_free(1) == 2);
    assert!(p.find_free(2) == 2);
    assert!(p.find_free(3) == 5);
    assert!(p.find_free(4) == 9);
    assert!(p.find_free(256) == 257);
    assert!(p.find_free(4096 * 8) == 0);
  }
}
