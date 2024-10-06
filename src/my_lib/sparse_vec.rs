use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use core::slice::Iter;

pub struct SparseVec<K, V> where K: Hash + Ord + Clone {
    map: HashMap<K, V>,
    keys: BinaryHeap<K>,
    cached_sorted_keys: Option<Vec<K>>,
}

impl<K, V> SparseVec<K, V> where K: Hash + Ord + Clone {
    pub fn new() -> Self {
        SparseVec { map: HashMap::new(), keys: BinaryHeap::new(), cached_sorted_keys: None }
    }
    
    pub fn insert(&mut self, key: K, value: V) {
        let key_copy = key.clone();
        self.map.insert(key, value);
        self.keys.push(key_copy);
        self.cached_sorted_keys = None;
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.keys.clear();
        self.cached_sorted_keys = None;
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn cache_keys(&mut self) {
        if self.cached_sorted_keys.is_none() {
            self.cached_sorted_keys = Some(self.keys.clone().into_sorted_vec());
        }
    }

    pub fn get_or_next(&self, key: &K) -> Result<Option<&V>, &'static str> {
        if self.map.is_empty() {
            return Ok(None);
        }

        let exact = self.map.get(key);
        if exact.is_some() {
            return Ok(exact);
        }

        if self.cached_sorted_keys.is_none() {
            return Err("Call cache_keys first");
        }
        let sorted_keys = self.cached_sorted_keys.as_ref().unwrap();

        match sorted_keys.binary_search(key) {
            Ok(_) => {
                panic!("sparse vec key was not found in map but was found in keys list");
            }
            Err(i) => {
                if i < sorted_keys.len() {
                    Ok(self.map.get(&sorted_keys[i]))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn iter_keys_ordered(&self) -> Result<Iter<K>, &'static str> {
        match &self.cached_sorted_keys {
            None => Err("Call cache_keys first"),
            Some(v) => Ok(v.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SparseVec;

    #[test]
    pub fn get_or_next() {
        let mut vec: SparseVec<u16, &'static str> = SparseVec::new();
        vec.insert(1, "a");
        vec.insert(5, "b");
        vec.insert(10, "c");

        vec.cache_keys();

        assert_eq!("a", *(vec.get_or_next(&0).unwrap().unwrap()));
        assert_eq!("a", *(vec.get_or_next(&1).unwrap().unwrap()));
        assert_eq!("b", *(vec.get_or_next(&2).unwrap().unwrap()));
        assert_eq!("b", *(vec.get_or_next(&5).unwrap().unwrap()));
        assert_eq!("c", *(vec.get_or_next(&6).unwrap().unwrap()));
        assert_eq!("c", *(vec.get_or_next(&10).unwrap().unwrap()));
        assert!(vec.get_or_next(&11).unwrap().is_none());
    }

    #[test]
    pub fn iter_keys_ordered() {
        let mut vec: SparseVec<u16, &'static str> = SparseVec::new();
        vec.insert(10, "c");
        vec.insert(1, "a");
        vec.insert(5, "b");

        vec.cache_keys();
        let mut iter = vec.iter_keys_ordered().unwrap();
        assert_eq!(1, *(iter.next().unwrap()));
        assert_eq!(5, *(iter.next().unwrap()));
        assert_eq!(10, *(iter.next().unwrap()));
        assert_eq!(None, iter.next());
    }
}
