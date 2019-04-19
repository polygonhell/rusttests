use fnv::FnvHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;
// use std::marker::PhantomData;
// use std::fmt::{Display, Debug};


#[allow(dead_code)]


pub trait JournalRep<K, V> {
  fn insert(&mut self, k: K, v: V) -> Option<V>;
  fn remove(&mut self, k: &K) -> Option<V>;
}

impl<K, V> JournalRep<K, V> for BTreeMap<K, V> where K: Ord {
  fn insert(&mut self, k: K, v: V) -> Option<V> {
    BTreeMap::insert(self, k, v)
  }

  fn remove(&mut self, k: &K)  -> Option<V> {
    BTreeMap::remove(self, k)
  }
}

impl<K, V> JournalRep<K, V> for FnvHashMap<K, V> where K: Eq+Hash {
  fn insert(&mut self, k: K, v: V) -> Option<V> {
    FnvHashMap::insert(self, k, v)
  }

  fn remove(&mut self, k: &K)  -> Option<V> {
    FnvHashMap::remove(self, k)
  }
}


#[derive(Debug)]
pub enum JournalEntry<K, V> {
  Add {key: K, value: V},
  Rm {key: K},
}

#[derive(Debug)]
pub struct Journal<K, V>(pub Vec<JournalEntry<K, V> > );

impl <K, V> Journal<K, V> where K : Copy, V: Copy {
  pub fn add(key: K, value: V) -> JournalEntry<K, V> {
    JournalEntry::Add {key: key, value: value}
  }
  pub fn rm(key: K) -> JournalEntry<K, V> {
    JournalEntry::Rm {key: key}
  }

  pub fn append(&mut self, entry: JournalEntry<K, V>) -> () {
    self.0.push(entry)
  }

  pub fn play<R>(&self, rep: &mut R) -> () where R : JournalRep<K, V> {
    self.0.iter().for_each(|x| {
      match x {
        JournalEntry::Add {key, value} => {rep.insert(*key, *value);},
        JournalEntry::Rm {key} => {rep.remove(key);}
      }
    })
  }
}



