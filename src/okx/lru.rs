use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

pub struct SimpleLru<K, V> {
  cache_size: usize,
  new_cache: HashMap<K, V>,
  old_cache: HashMap<K, V>,
}

impl<K, V> SimpleLru<K, V>
where
  K: Eq + Hash,
{
  pub fn new(cache_size: usize) -> SimpleLru<K, V> {
    Self {
      cache_size,
      new_cache: HashMap::with_capacity(cache_size),
      old_cache: HashMap::new(),
    }
  }

  pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq,
  {
    if let Some(v) = self.new_cache.get(key) {
      Some(v)
    } else {
      self.old_cache.get(key)
    }
  }

  pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
  where
    K: Borrow<Q>,
    Q: Hash + Eq,
  {
    if self.new_cache.contains_key(key) {
      true
    } else {
      self.old_cache.contains_key(key)
    }
  }

  pub fn insert(&mut self, key: K, value: V) -> Option<V> {
    self.new_cache.insert(key, value)
  }

  pub fn refresh(&mut self) {
    if self.new_cache.len() >= self.cache_size {
      self.old_cache.clear();
      mem::swap(&mut self.new_cache, &mut self.old_cache);
    }
  }

  pub fn len(&self) -> usize {
    self.old_cache.len() + self.new_cache.len()
  }
}

#[cfg(test)]
mod tests {
  use crate::okx::lru::SimpleLru;

  #[test]
  fn lru_test() {
    let mut lru = SimpleLru::new(2);
    lru.insert(1, 11);
    lru.insert(2, 22);
    assert!(lru.get(&1).is_some());
    assert!(lru.get(&2).is_some());
    assert!(lru.contains(&1));
    assert!(lru.contains(&2));
    assert_eq!(2, lru.len());
    lru.refresh();

    lru.insert(3, 33);
    lru.insert(4, 44);
    assert!(lru.contains(&1));
    assert!(lru.contains(&2));
    assert!(lru.contains(&3));
    assert!(lru.contains(&4));
    assert!(lru.get(&3).is_some());
    assert!(lru.get(&4).is_some());
    assert_eq!(4, lru.len());

    lru.refresh();
    lru.insert(5, 55);
    assert!(!lru.contains(&1));
    assert!(!lru.contains(&2));
    assert!(lru.contains(&3));
    assert!(lru.contains(&4));
    assert!(lru.contains(&5));
    assert!(lru.get(&1).is_none());
    assert!(lru.get(&2).is_none());
    assert!(lru.get(&3).is_some());
    assert!(lru.get(&4).is_some());
    assert!(lru.get(&5).is_some());
    assert_eq!(3, lru.len());

    lru.refresh();
    lru.insert(6, 66);
    assert!(lru.contains(&3));
    assert!(lru.contains(&4));
    assert!(lru.contains(&5));
    assert!(lru.contains(&6));
    assert!(lru.get(&3).is_some());
    assert!(lru.get(&4).is_some());
    assert!(lru.get(&5).is_some());
    assert!(lru.get(&6).is_some());
    assert_eq!(4, lru.len());

    lru.refresh();
    lru.insert(7, 77);
    assert!(!lru.contains(&3));
    assert!(!lru.contains(&4));
    assert!(lru.contains(&5));
    assert!(lru.contains(&6));
    assert!(lru.contains(&7));
    assert!(lru.get(&3).is_none());
    assert!(lru.get(&4).is_none());
    assert!(lru.get(&5).is_some());
    assert!(lru.get(&6).is_some());
    assert!(lru.get(&7).is_some());
    assert_eq!(3, lru.len());

    lru.refresh();
    assert_eq!(55, *lru.get(&5).unwrap());
    assert_eq!(66, *lru.get(&6).unwrap());
    assert_eq!(77, *lru.get(&7).unwrap());
  }

  #[test]
  fn lru_swap_test() {
    const CACHE_SIZE: usize = 10000000;
    let mut lru = SimpleLru::new(CACHE_SIZE);
    for i in 0..CACHE_SIZE {
      lru.insert(i, i);
    }
    assert_eq!(CACHE_SIZE, lru.len());
    lru.refresh();
    assert_eq!(CACHE_SIZE, lru.len());

    for i in 0..CACHE_SIZE {
      lru.insert(i, i);
    }
    assert_eq!(2 * CACHE_SIZE, lru.len());
    lru.refresh();
    assert_eq!(CACHE_SIZE, lru.len());
  }
}
