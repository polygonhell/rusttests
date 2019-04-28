#![allow(dead_code)]
use fnv::{FnvHashMap, FnvHasher};
use std::hash::{Hash, Hasher};


pub struct Dictionary<A> {
  entries: Vec<A>,
  index: Option<FnvHashMap<u64, Vec<usize>>>, 
}



// TODO implement over storage
impl<A : Copy + Hash + Eq> Dictionary<A> {

  pub fn new() -> Dictionary<A> {
    Dictionary {
      entries : Vec::new(),
      index: None,
    }
  }

  pub fn add(&mut self, val: &A) -> usize {
    let mut hasher:FnvHasher = FnvHasher::default();


    match &self.index {
      None => { 
        let map = FnvHashMap::default();
        self.index = Some(map);
      }
      Some(_val) => ()
    };

    let index = self.index.as_mut().unwrap();

    val.hash(&mut hasher);
    let hash = hasher.finish();
    match index.get(&hash) {
      None => {}
      Some(idxs) => {
        for i in idxs {
          if self.entries[*i] == *val { return *i; }
        };
      }
    }

    let i = self.entries.len();
    self.entries.push(*val);
    index.insert(hash, vec![i]);
    i

  }

  pub fn get(&self, i: usize) -> &A {
    & self.entries[i]
  }
}


#[cfg(test)]
pub mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  pub fn add() {
    let mut d = Dictionary::<&str>::new();
    assert!(d.add(&"This is a test") == 0);
    assert!(d.add(&"And another test") == 1);
    assert!(d.add(&"This is a test") == 0);
    assert!(d.add(&"And another test") == 1);
    assert!(d.add(&"And a third test") == 2);
  }
}