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
pub struct Database {
  mmap: MmapMut,
}

#[derive(Debug)]
pub enum DbError {
  Io(std::io::Error),
  Err(String),
}

use std::fs::{File, OpenOptions};
use std::path::Path;

use memmap::MmapMut;

static HEADER: Header = Header {
  version: Version {
    major_version: 0,
    minor_version: 0,
    patch_level: 0,
    dummy: 0,
  },
  page_size_shift: 12,
  free_list: 1,
  table_index: 2,
  pages: 256,
};

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
      .set_len((HEADER.pages as u64) << HEADER.page_size_shift)
      .map_err(DbError::Io)?;
    let mmap = unsafe { MmapMut::map_mut(&file) }.map_err(DbError::Io)?;

    let mut db = Database {
      mmap : mmap,
    };

    let hdr = db.header();
    *hdr = HEADER;

    // TODO: initalize the base tables and the free list
    Ok(db)
  }

  fn header(&mut self) -> &mut Header {
    unsafe {
      &mut (*(self.mmap.as_mut_ptr() as *mut Header))
    }
  }
}
