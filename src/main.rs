#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::BTreeMap;

use std::fmt;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;

mod bit_array;
mod dictionary_old;
mod dictionary;
mod database;
mod paged_vector;


fn write_file() -> Result<(), std::io::Error> {
  let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("foo.txt")?;

  for v in 0..10 {
    let buff = [v as u8; 4096];
    file.write_all(& buff)?;
  }

  Ok(())
}


// fn map_file() -> Result<(), std::io::Error> {
//   use mmap_fixed::{MemoryMap, MapOption};
//   use std::os::unix::io::AsRawFd;

//   let file = OpenOptions::new()
//             .read(true)
//             .open("foo.txt")?;

//   let handle = file.as_raw_fd();

//   let options = [ MapOption::MapReadable, MapOption::MapFd(handle) ];
//   let map1 = MemoryMap::new(65536*10, &options).unwrap();

//   let options = [ MapOption::MapReadable, 
//                   MapOption::MapAddr(unsafe{ map1.data().offset(4096) }),
//                   MapOption::MapFd(handle),
//                   MapOption::MapOffset(4096*3)
//                   ];
//   let map2 = MemoryMap::new(4096, &options).unwrap();


//   println!("map1 addr = {:?}, length = {}", map1.data(), map1.len());
//   println!("map2 addr = {:?}, length = {}", map2.data(), map2.len());

//   let slice = unsafe { std::slice::from_raw_parts(map1.data(), map1.len()) };

//   println!("map1 0 = {}, 4096 = {}, 8192 = {}", slice[0], slice[4096], slice[8192]);




//   Ok(())
// }


use database::Database;
use journal::Journal;

#[derive(Debug)]
enum AppError {
  DbError(database::DbError),
}

fn main() -> Result<(), AppError> {

  // let map1 = unsafe { map::map(& file); }
  // write_file()?;
  println!("3 = {}", 3u8.leading_zeros());
  println!("5 = {}", 5u8.leading_zeros());
  println!("9 = {}", 9u8.leading_zeros());

  let _db = Database::new("system.db").map_err(AppError::DbError)?;






  Ok(())
}
