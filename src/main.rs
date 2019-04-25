use fnv::FnvHashMap;
use std::collections::BTreeMap;

use std::fmt;
use std::fs::File;
use memmap::{Mmap};

mod journal;
use journal::*;

#[derive(Debug)]
struct Foo {
  a: i32,
  b: i32,
}

impl fmt::Display for Foo {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({:X}, {:X})", self.a, self.b)
  }

}



fn main() -> Result<(), std::io::Error> {
  let mut journal : Journal<&str, &str> = Journal::new();
  journal.append(Journal::add("hello", "world"));
  journal.append(Journal::add("hello2", "world2"));
  journal.append(Journal::add("hello3", "world3"));
  journal.append(Journal::rm("hello2"));
  journal.append(Journal::add("hello4", "something"));

  let mut map : BTreeMap<&str, &str> = BTreeMap::new();
  journal.play(&mut map);

  let mut map2 : FnvHashMap<&str, &str> = FnvHashMap::default();
  journal.play(&mut map2);

  println!("journal {:?}", journal);
  println!("map {:?}", map);
  println!("map2 {:?}", map2);

  let file = File::open("cargo.lock")?;
  let mmap = unsafe { Mmap::map(&file)? };

  let bar = &mmap[8..];
  let bar2 = unsafe {
    & *(bar as *const [u8] as *const Foo)
  };


  println!("file length  = {:?}", mmap.len());
  println!("Foo  = {}", bar2);

  Ok(())
}
