use fnv::FnvHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::iter::FromIterator;

pub trait JournalRep<K, V> {
  fn insert(&mut self, k: K, v: V) -> Option<V>;
  fn remove(&mut self, k: &K) -> Option<V>;
}

impl<K, V> JournalRep<K, V> for BTreeMap<K, V>
where
  K: Ord,
{
  fn insert(&mut self, k: K, v: V) -> Option<V> {
    BTreeMap::insert(self, k, v)
  }

  fn remove(&mut self, k: &K) -> Option<V> {
    BTreeMap::remove(self, k)
  }
}

impl<K, V> JournalRep<K, V> for FnvHashMap<K, V>
where
  K: Eq + Hash,
{
  fn insert(&mut self, k: K, v: V) -> Option<V> {
    FnvHashMap::insert(self, k, v)
  }

  fn remove(&mut self, k: &K) -> Option<V> {
    FnvHashMap::remove(self, k)
  }
}

#[derive(Debug, Clone)]
pub enum JournalEntry<K, V> {
  Add { key: K, value: V },
  Rm { key: K },
}

#[derive(Debug)]
pub struct Journal<K, V>(pub Vec<JournalEntry<K, V>>);

impl<K, V> Journal<K, V>
where
  K: Copy,
  V: Copy,
{
  pub fn new() -> Journal<K, V> {
    Journal(Vec::new())
  }

  #[allow(dead_code)]
  pub fn from_iter<I: IntoIterator<Item = JournalEntry<K, V>>>(iter: I) -> Journal<K, V> {
    Journal(Vec::from_iter(iter))
  }

  pub fn add(key: K, value: V) -> JournalEntry<K, V> {
    JournalEntry::Add {
      key: key,
      value: value,
    }
  }
  pub fn rm(key: K) -> JournalEntry<K, V> {
    JournalEntry::Rm { key: key }
  }

  pub fn append(&mut self, entry: JournalEntry<K, V>) -> () {
    self.0.push(entry)
  }

  pub fn play<R>(&self, rep: &mut R) -> ()
  where
    R: JournalRep<K, V>,
  {
    self.0.iter().for_each(|x| match x {
      JournalEntry::Add { key, value } => {
        rep.insert(*key, *value);
      }
      JournalEntry::Rm { key } => {
        rep.remove(key);
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert() {
    let mut journal: Journal<u32, &str> = Journal::new();
    journal.append(Journal::add(0, "hello"));
    assert!(journal.0.len() == 1);
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    journal.play(&mut map);
    assert!(map.len() == 1);
    assert!(map[&0] == "hello");
    // playing the same journal should have same results
    journal.play(&mut map);
    assert!(map.len() == 1);
    assert!(map[&0] == "hello");
  }

  #[test]
  fn inserts() {
    let mut journal: Journal<u32, &str> = Journal::new();
    journal.append(Journal::add(1, "hello"));
    journal.append(Journal::add(0, "world"));
    assert!(journal.0.len() == 2);
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    journal.play(&mut map);
    assert!(map.len() == 2);
    assert!(map[&1] == "hello");
    assert!(map[&0] == "world");
  }

  #[test]
  fn mixed() {
    let mut journal: Journal<u32, &str> = Journal::new();
    journal.append(Journal::add(1, "hello"));
    journal.append(Journal::add(0, "world"));
    journal.append(Journal::rm(0));
    assert!(journal.0.len() == 3);
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    journal.play(&mut map);
    assert!(map.len() == 1);
    assert!(map[&1] == "hello");
  }

  #[test]
  fn readd() {
    let journal: Journal<u32, &str> = Journal::from_iter(vec![
      Journal::add(0, "foo"),
      Journal::add(1, "bar"),
      Journal::add(2, "baz"),
      Journal::rm(0),
      Journal::add(2, "cat"),
    ]);
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    journal.play(&mut map);
    assert!(map == BTreeMap::from_iter(vec![(1, "bar"), (2, "cat")]))
  }

}
