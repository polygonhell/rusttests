// Basically not quite a B+ Tree
// an append only structure allocated per page, with a tiered index
// Fills like this

// Page1
// Index1 => Page1  |Page2 | ...
// Page2 onwards being allocated as previous page is filled
// Random access is then available through just the index depth
// The index reduces the number of disk page reads when fetching the required page

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::database::{MemoryPageProvider, PageProvider};
use std::fmt::Debug;

const PAGE_SIZE_SHIFT: u8 = 12;
const PAGE_SIZE: usize = 1 << PAGE_SIZE_SHIFT;

#[derive(Debug)]
#[repr(C)]
struct PageHeader {
  version: u8,
  depth: u8, // Stores the tree depth at the root set to 0 if a leaf
  entries: u16,
  next: u32, // Adjacency Chain like
}

impl PageHeader {
  fn is_leaf(&self) -> bool {
    self.depth == 0
  }
}

const EMPTY_HEADER: PageHeader = PageHeader {
  version: 0,
  depth: 0,
  entries: 0,
  next: 0,
};

#[repr(C)]
pub struct Page {
  header: PageHeader,
  data: [u32; 1],
}

pub struct PageRef<'a, T> {
  header: &'a PageHeader,
  data: &'a [T]
}

pub struct MutPageRef<'a, T> {
  header: &'a mut PageHeader,
  data: &'a mut [T]
}

impl Page {
  const fn capacity<T>() -> usize {
    (PAGE_SIZE - std::mem::size_of::<PageHeader>()) / std::mem::size_of::<T>()
  }

  fn mut_pref<T>(&mut self) -> MutPageRef<T> {
    let entries = self.header.entries as usize;
    MutPageRef::<T> {
      header: &mut self.header,
      data: unsafe { std::slice::from_raw_parts_mut(&mut self.data[0] as *mut u32 as *mut T, entries) },
    }
  }

  fn pref<T>(&self) -> PageRef<T> {
    let entries = self.header.entries as usize;
    PageRef::<T> {
      header: &self.header,
      data: unsafe { std::slice::from_raw_parts(&self.data[0] as *const u32 as *const T, entries) },
    }
  }

  fn init(&mut self) {
    self.header = EMPTY_HEADER;
  }
}

fn rotate_node(page_indices: &[u32], pp: &mut PageProvider, depth: u8) -> u32 {
  if page_indices.len() <= Page::capacity::<u32>() {
    // Create a new index node
    let new_index = pp.alloc(1)[0];
    let (_, new_page) = pp.mut_page(new_index);
    new_page.header = EMPTY_HEADER;
    new_page.header.depth = depth;
    new_page.header.entries = page_indices.len() as u16;
    let new_page_ref = new_page.mut_pref::<u32>();
    new_page_ref.data.copy_from_slice(page_indices);
    new_index
  } else {
    panic!("Not Implemented: insert requires multiple rotations")
  }
}

fn append<T: Debug>(
  page_index: u32,
  v: &T,
  leaf_fn: &Fn(&mut Page, &T, &mut PageProvider) -> Vec<u32>,
  pp: &mut PageProvider,
) -> u32 {
  // Root case
  let (pp, page) = pp.mut_page(page_index);
  let mut new_pages = if page.header.is_leaf() {
    // Root of tree is a leaf no where to insert the returned page, so increase the tree depth
    leaf_fn(page, v, pp)
  } else {
    append_i(page.header.depth, page, v, leaf_fn, pp)
  };

  if !new_pages.is_empty() {
    new_pages.insert(0, page_index);
    println!("Rotation to depth {}", page.header.depth + 1);
    rotate_node(&new_pages, pp, page.header.depth + 1)
  } else {
    page_index
  }
}

fn append_i<T: Debug>(
  parent_depth: u8,
  page: &mut Page,
  v: &T,
  leaf_fn: &Fn(&mut Page, &T, &mut PageProvider) -> Vec<u32>,
  pp: &mut PageProvider,
) -> Vec<u32> {
  if page.header.is_leaf() {
    // index will deal with the rotation if it's full
    leaf_fn(page, v, pp)
  } else {
    // Proceed down to next level
    let page_ref = page.mut_pref::<u32>();
    let page_data = page_ref.data;
    let next_page_index = page_data[page_ref.header.entries as usize - 1];
    let (pp, next_page) = pp.mut_page(next_page_index);
    let mut new_pages = append_i(page_ref.header.depth, next_page, v, leaf_fn, pp);

    if !new_pages.is_empty() {
      if next_page.header.depth + 1 != page_ref.header.depth {
        new_pages.insert(0, next_page_index);
        let new_index = rotate_node(&new_pages, pp, next_page.header.depth + 1);
        page_data[page_data.len() as usize - 1] = new_index;
        vec![]
      } else {
        let free_slots = Page::capacity::<u32>() - page.header.entries as usize;
        let to_copy = std::cmp::min(free_slots, new_pages.len());
        let entries = page.header.entries;
        page.header.entries += to_copy as u16;
        let page_ref = page.mut_pref::<u32>();
        page_ref.data[entries as usize..entries as usize + to_copy]
          .copy_from_slice(&new_pages[..to_copy]);
        let rem = &new_pages[to_copy..];
        rem.to_vec()
      }
    } else {
      new_pages
    }
  }
}

// Returns a none 0 return if a new page was allocated and needs to be added to the parent index
fn append_u32(page: &mut Page, v: &u32, pp: &mut PageProvider) -> Vec<u32> {
  // If it fits
  if (page.header.entries as usize) < Page::capacity::<u32>() {
    unsafe {
      let ptr = (&page.data[0] as *const u32).offset(page.header.entries as isize) as *mut u32;
      *ptr = *v;
    }
    page.header.entries += 1;
    vec![]
  } else {
    // Append a new page
    let new_pages = pp.alloc(1);
    assert!(!new_pages.is_empty());
    let (pp, new_page) = pp.mut_page(new_pages[0]);
    new_page.init();
    page.header.next = new_pages[0];
    // Stick the value in the new page
    append_u32(new_page, v, pp);
    new_pages
  }
}

// Shared by get and iterator code
fn page_ref<T>(page_index: u32, index: usize, pp: &PageProvider) -> (u32, &Page, usize) {
  // Get the current page
  let page = pp.page(page_index);
  if page.header.is_leaf() {
    (page_index, page, index)
  } else {
    let leaf_capacity = Page::capacity::<T>();
    let index_capacity = Page::capacity::<u32>();
    let page_contains = index_capacity.pow(page.header.depth as u32 - 1) * leaf_capacity;
    let page_data = page.pref::<u32>().data;
    let next_page = page_data[index / page_contains];
    let next_index = index % page_contains;
    page_ref::<T>(next_page, next_index, pp)
  }
}

fn get<'a, T: Debug>(
  page_index: u32,
  index: usize,
  leaf_fn: &Fn(&'a Page, usize) -> &'a T,
  pp: &'a PageProvider,
) -> &'a T {
  let (page_index, page, index) = page_ref::<T>(page_index, index, pp);
  leaf_fn(page, index)
}

fn get_u32<'a>(page: &'a Page, index: usize) -> &'a u32 {
  assert!(index < page.header.entries as usize);
  let page_data = page.pref::<u32>().data;
  &page_data[index]
}

// Walking the index is common regardless of type
trait PagedVectorFns<'a, T> {
  fn append(&mut self, v: &T);
  fn get(&self, i: usize) -> &T;
  fn iter_from(&'a self, i: usize) -> PagedVectorIterator<T>;
  // fn get_slice(&self, i: usize, len: usize) -> &[T];
}

struct PagedVector<'a> {
  db: &'a mut PageProvider,
  entry_page: u32,
}

impl<'a> PagedVectorFns<'a, u32> for PagedVector<'a> {
  fn append(&mut self, v: &u32) {
    self.entry_page = append(self.entry_page, v, &append_u32, self.db);
  }

  fn get(&self, i: usize) -> &u32 {
    get(self.entry_page, i, &get_u32, self.db)
  }

  fn iter_from(&'a self, i: usize) -> PagedVectorIterator<u32> {
    let (_page_index, page, index) = page_ref::<u32>(self.entry_page, i, self.db);
    PagedVectorIterator {
      vector: self,
      page: page,
      offset: index,
      _dummy: std::marker::PhantomData,
    }
  }
}

struct PagedVectorIterator<'a, T> {
  vector: &'a PagedVector<'a>,
  page: &'a Page,
  offset: usize,
  _dummy: std::marker::PhantomData<T>,
}

impl<'a> Iterator for PagedVectorIterator<'a, u32> {
  type Item = u32;
  fn next(&mut self) -> Option<u32> {
    let page_data = self.page.pref::<u32>().data;

    if self.offset < page_data.len() {
      let val = page_data[self.offset];
      self.offset += 1;
      Some(val)
    } else {
      if self.page.header.next != 0 {
        self.page = self.vector.db.page(self.page.header.next);
        self.offset = 0;
        let page_data = self.page.pref::<u32>().data;
        let val = page_data[self.offset];
        self.offset += 1;
        Some(val)
      } else {
        None
      }
    }
  }
}

#[cfg(test)]
pub mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  pub fn add() {
    let mut pp = MemoryPageProvider::new();
    let root = pp.alloc(1)[0];
    let (pp, page) = pp.mut_page(root);
    page.header = EMPTY_HEADER;

    let mut p = PagedVector {
      db: pp,
      entry_page: root,
    };

    p.append(&1);
    p.append(&2);
    p.append(&3);

    assert!(page.header.entries == 3);
    unsafe {
      let ptr = &page.data[0] as *const u32;
      assert!(*ptr == 1);
      assert!(*ptr.offset(1) == 2);
      assert!(*ptr.offset(2) == 3);
    }

    assert!(*p.get(0) == 1);
    assert!(*p.get(1) == 2);
    assert!(*p.get(2) == 3);

    let mut count: u32 = 1;
    p.iter_from(0).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 4);
  }

  #[test]
  pub fn add_some() {
    let mut pp = MemoryPageProvider::new();
    let root = pp.alloc(1)[0];
    let (pp, page) = pp.mut_page(root);
    page.header = EMPTY_HEADER;

    let mut p = PagedVector {
      db: pp,
      entry_page: root,
    };

    for i in 0..1024 {
      p.append(&(i as u32));
    }

    let (pp, page) = p.db.mut_page(p.entry_page);

    println!("page.header = {:?}", page.header);
    assert!(page.header.depth == 1);
    assert!(page.header.entries == 2);
    let page_ptr = &page.data[0] as *const u32;
    let (pp, page1) = p.db.mut_page(unsafe { *page_ptr });
    let page2_index = unsafe { *page_ptr.offset(1) };
    assert!(page1.header.next == page2_index);
    let (pp, page2) = pp.mut_page(page2_index);
    assert!(page1.header.depth == 0);
    assert!(page1.header.entries == Page::capacity::<u32>() as u16);
    for i in 0..Page::capacity::<u32>() {
      unsafe {
        let ptr = (&page1.data[0] as *const u32).offset(i as isize);
        assert!(*ptr == i as u32)
      }
    }

    assert!(page2.header.depth == 0);
    assert!(page2.header.entries == (1024 - Page::capacity::<u32>()) as u16);
    for i in Page::capacity::<u32>()..1024 {
      unsafe {
        let ptr = (&page2.data[0] as *const u32).offset((i - Page::capacity::<u32>()) as isize);
        assert!(*ptr == i as u32)
      }
    }

    let mut count: u32 = 0;
    p.iter_from(0).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 1024);

    count = 500;
    p.iter_from(count as usize).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 1024);
  }

  #[test]
  pub fn add_lots() {
    let mut pp = MemoryPageProvider::new();
    let root = pp.alloc(1)[0];
    let (pp, page) = pp.mut_page(root);
    page.header = EMPTY_HEADER;

    let mut p = PagedVector {
      db: pp,
      entry_page: root,
    };

    for i in 0..4000000 {
      p.append(&(i as u32));
    }

    let (pp, page) = p.db.mut_page(p.entry_page);
    println!("Final Page header = {:?}", page.header);

    for i in 0..4000000 {
      assert!(*p.get(i) == i as u32);
    }

    let mut count: u32 = 0;
    p.iter_from(0).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 4000000);

    count = 1234567;
    p.iter_from(count as usize).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 4000000);
  }

  #[test]
  // #[ignore] // Takes way to long to run, but necessary
  pub fn add_even_more() {
    let mut pp = MemoryPageProvider::new();
    let root = pp.alloc(1)[0];
    let (pp, page) = pp.mut_page(root);
    page.header = EMPTY_HEADER;

    let mut p = PagedVector {
      db: pp,
      entry_page: root,
    };

    // Should result in an extra level
    for i in 0..2000000000 {
      p.append(&(i as u32));
    }

    let page = p.db.page(p.entry_page);
    println!(
      "Final Page header for really big vector = {:?}",
      page.header
    );

    let mut count: u32 = 0;
    p.iter_from(0).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 2000000000);

    for i in 0..2000000000 {
      assert!(*p.get(i) == i as u32);
    }
  }

}
