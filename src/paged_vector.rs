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
  data: &'a [T],
}

pub struct MutPageRef<'a, T> {
  header: &'a mut PageHeader,
  data: &'a mut [T],
}

impl Page {
  const fn capacity<T>() -> usize {
    (PAGE_SIZE - std::mem::size_of::<PageHeader>()) / std::mem::size_of::<T>()
  }

  fn mut_pref<T>(&mut self) -> MutPageRef<T> {
    let entries = self.header.entries as usize;
    MutPageRef::<T> {
      header: &mut self.header,
      data: unsafe {
        std::slice::from_raw_parts_mut(&mut self.data[0] as *mut u32 as *mut T, entries)
      },
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

#[inline(always)]
fn last_page(page_index: u32, pp: &PageProvider) -> u32 {
  let page = pp.page(page_index);
  if page.header.depth == 0 {
    page_index
  } else {
    let data = page.pref::<u32>().data;
    last_page(data[page.header.entries as usize - 1], pp)
  }
}

fn rotate_slice<'a, T: Debug + Copy>(
  page_index: u32,
  page: &mut Page,
  v: &'a [T],
  pp: &mut PageProvider,
  depth: u8,
) -> (u32, &'a [T]) {
  let max_pages = Page::capacity::<u32>() - 1;
  let page_capacity = Page::capacity::<T>();
  let new_pages = std::cmp::min(max_pages, (v.len() + page_capacity - 1)/ page_capacity);
  let to_take = std::cmp::min(new_pages * page_capacity, v.len());
  let new_page_data = (&v[..to_take]).chunks(page_capacity);

  // Create the pages that can be accomodated in the new root page
  let mut page_indices = vec![page_index];
  let last_page_index = last_page(page_index, pp);
  let (pp, last_page) = pp.mut_page(last_page_index);
  let (pp, _) = new_page_data.fold((pp, last_page), |(pp, prev_page): (&mut PageProvider, &mut Page), x: &[T]| {
    let page_index = pp.alloc(1)[0];
    prev_page.header.next = page_index;         // Link to previous page
    let (pp, page) = pp.mut_page(page_index);
    page.header = EMPTY_HEADER;
    page.header.entries = x.len() as u16;
    let page_ref = page.mut_pref::<T>();
    page_ref.data.copy_from_slice(x);
    page_indices.push(page_index);
    (pp, page)
  });

  let new_index = pp.alloc(1)[0];
  let (_, new_page) = pp.mut_page(new_index);
  new_page.header = EMPTY_HEADER;
  new_page.header.depth = depth;
  new_page.header.entries = page_indices.len() as u16;
  let new_page_ref = new_page.mut_pref::<u32>();
  new_page_ref.data.copy_from_slice(page_indices.as_slice());

  let residual = &v[to_take..];
  (new_index, residual)
}

fn append_slice<'a, T: Debug + Copy>(
  page_index: u32,
  v: &'a [T],
  pp: &mut PageProvider,
) -> u32 {
  // Root case
  let (pp, page) = pp.mut_page(page_index);
  let remaining_values = if page.header.is_leaf() {
    // Root of tree is a leaf no where to insert the returned page, so increase the tree depth
    append_slice_leaf(page, v, pp)
  } else {
    append_slice_i(page.header.depth, page, v, pp)
  };

  // If everything is full rotate existing tree left and create a new right tree
  if !remaining_values.is_empty() {
    println!("Rotation to depth {}", page.header.depth + 1);
    let (page_index, remaining_values) = rotate_slice(
      page_index,
      page,
      remaining_values,
      pp,
      page.header.depth + 1,
    );
    // If rotate didn't create enough space insert what's left into the newly rotated tree
    if !remaining_values.is_empty() {
      append_slice(page_index, remaining_values, pp)
    } else {
      page_index
    }
  } else {
    page_index
  }
}


fn append_slice_i<'a, T: Debug+Copy>(
  parent_depth: u8,
  page: &mut Page,
  v: &'a [T],
  pp: &mut PageProvider,
) -> &'a [T] {
  if page.header.is_leaf() {
    // index will deal with the rotation if it's full
    append_slice_leaf(page, v, pp)
  } else {
    // Proceed down to next level
    let page_ref = page.mut_pref::<u32>();
    let page_data = page_ref.data;
    let next_page_index = page_data[page_ref.header.entries as usize - 1];
    let (pp, next_page) = pp.mut_page(next_page_index);
    let residual = append_slice_i(page_ref.header.depth, next_page, v, pp);
    if !residual.is_empty() {
      // if the tree is not full
      if next_page.header.depth + 1 != page_ref.header.depth {
        let (new_index, residual) = rotate_slice(next_page_index, next_page, residual, pp, next_page.header.depth + 1);
        page_data[page_data.len() as usize - 1] = new_index;
        // attempt reinsert since subtrees may not be full
        append_slice_i(parent_depth, page, residual, pp)
      } else {
        // Append a new page to this index layer if it's not full
        let entries = page.header.entries as usize;
        if entries == Page::capacity::<u32>() {
          residual
        } else {
          let page_capacity = Page::capacity::<T>();
          let to_take = std::cmp::min(page_capacity, residual.len());
          let new_page_index = pp.alloc(1)[0];
          let (pp, new_page) = pp.mut_page(new_page_index);
          new_page.header = EMPTY_HEADER;
          new_page.header.entries = to_take as u16;
          let new_page_ref = new_page.mut_pref::<T>();
          new_page_ref.data.copy_from_slice(&residual[..to_take]);
          
          // Link to previous last page - next page is now the previous page
          let last_page_index = last_page(next_page_index, pp);
          let (pp, last_page) = pp.mut_page(last_page_index);
          last_page.header.next = new_page_index;          // Link to previous page

          // Add the new page to the index
          page.header.entries += 1 as u16;
          let page_ref = page.mut_pref::<u32>();
          page_ref.data[entries] = new_page_index;

          let residual = &residual[to_take..];
          if residual.is_empty() || page.header.depth == 1 {
            residual
          } else {
            // retry at this level if the new page didn't result in a full tree
            append_slice_i(parent_depth, page, residual, pp)
          }
        }
      }
    } else {
      residual
    }
  }
}


fn append_slice_leaf<'a, T: Debug+Copy>(page: &mut Page, v: &'a [T], pp: &mut PageProvider) -> &'a [T] {
  let page_capacity = Page::capacity::<T>();
  let entries = page.header.entries as usize;
  let free_entries = page_capacity - entries;
  if free_entries == 0 {
    v
  } else {
    // copy what we can into the page
    let to_take = std::cmp::min(v.len(), free_entries);
    page.header.entries += to_take as u16;
    let page_ref = page.mut_pref::<T>();
    (page_ref.data[entries..]).copy_from_slice(&v[..to_take]);
    let residual = &v[to_take..];
    residual
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
  pp: &'a PageProvider,
) -> &'a T {
  let (page_index, page, index) = page_ref::<T>(page_index, index, pp);
  let data = page.pref::<T>().data;
  &data[index]
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
    // self.entry_page = append(self.entry_page, v, &append_u32, self.db);
    self.entry_page = append_slice(self.entry_page, &[*v], self.db);
  }

  fn get(&self, i: usize) -> &u32 {
    get(self.entry_page, i, self.db)
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
  // #[ignore] // Takes way to long to run, but necessary
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
  // #[ignore] // Takes way to long to run, but necessary
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
    println!("Final Page header = {:?}", page.header);

    let entries = page.header.entries;
    let pageref = page.mut_pref::<u32>();
    pageref.data.iter().for_each(|&index| {
      let (pp, page) = pp.mut_page(index);
      println!("header = {:?}", page.header);
    });
    assert!(entries == 2);


    for i in 0..1024 {
      assert!(*p.get(i) == i as u32);
    }
  }

  #[test]
  // #[ignore] // Takes way to long to run, but necessary
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


    let entries = page.header.entries;
    let pageref = page.mut_pref::<u32>();
    pageref.data.iter().for_each(|&index| {
      let (pp, page) = pp.mut_page(index);
      println!("header = {:?}", page.header);
    });


    for i in 0..4000000 {
      assert!(*p.get(i) == i as u32);
    }

    let mut count: u32 = 0;
    p.iter_from(0).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    println!("count = {:?}", count);
    assert!(count == 4000000);

    count = 1234567;
    p.iter_from(count as usize).for_each(|x| {
      assert!(x == count);
      count += 1;
    });
    assert!(count == 4000000);
  }

  #[test]
  #[ignore] // Takes way to long to run, but necessary
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
    for i in 0..4000000000u64 {
      p.append(&(i as u32));
    }

    let page = p.db.page(p.entry_page);
    println!(
      "Final Page header for really big vector = {:?}",
      page.header
    );

    let mut count: u64 = 0;
    p.iter_from(0).for_each(|x| {
      assert!(x == count as u32);
      count += 1;
    });
    assert!(count == 4000000000u64);

    for i in 0..4000000000usize {
      assert!(*p.get(i) == i as u32);
    }
  }

}
