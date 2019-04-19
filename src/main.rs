use fnv::FnvHashMap;
use std::collections::BTreeMap;


mod journal;
use journal::*;

fn main() {
  let mut journal : Journal<&str, &str> = Journal(Vec::new());
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
}
