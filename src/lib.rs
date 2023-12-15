#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeMap, Serializer};
#[cfg(feature = "serde")]
use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};
#[cfg(feature = "serde")]
use std::marker::PhantomData;
#[cfg(feature = "serde")]
use std::fmt;

struct Entry<K, V> {
    key: K,
    val: V,
}

/// An `RegistOrderMap` is like a `std::collections::HashMap`,
/// but it is sorted according to the key in descending order.
/// The `RegistOrderMap` is a `HashMap` with guaranteed registration order.
/// 
/// * Examples
/// ```rust
/// ```
pub struct RegistOrderMap<K, V> {
    entries: Vec<Entry<K, V>>,
}

impl<K, V> RegistOrderMap<K, V> {
    pub fn new() -> Self {
        Default::default()
    }
    fn find(&self, k: &K) -> Option<usize>
    where
        K: Eq
    {
        self.entries.iter().position(|e| e.key == *k)
    }
    pub fn get(&self, k: &K) -> Option<&V>
    where
        K: Eq
    {
        match self.find(k) {
            Some(i) => Some(&self.entries[i].val),
            None => None,
        }
    }
    pub fn insert(&mut self, k: K, v: V)
    where
        K: Eq
    {
        match self.find(&k) {
            None => self.entries.push(Entry { key: k, val: v}),
            Some(i) => self.entries[i].val = v,
        }
    }
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.entries.iter(),
        }
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }
}

impl<K, V> Default for RegistOrderMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for RegistOrderMap<K, V>
where
    K: Eq + Copy,
    V: Copy,
{
    fn from(arr: [(K, V); N]) -> Self {
        Self {
            entries: arr.iter().map(|e| Entry{ key:  e.0, val: e.1 }).collect(),
        }
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    inner: std::slice::Iter<'a, Entry<K, V>>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V>
where
    K: Eq,
{
    type Item = (&'a K,  &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(entry) => Some((&entry.key, &entry.val))
        }
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize  for RegistOrderMap<K, V>
where
    K: Serialize + Eq,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
struct RegistOrderMapVisitor<K, V> {
    marker: PhantomData<fn() -> RegistOrderMap<K, V>>
}

#[cfg(feature = "serde")]
impl<K, V> RegistOrderMapVisitor<K, V> {
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Visitor<'de> for RegistOrderMapVisitor<K, V>
where
    K: Deserialize<'de> + Eq,
    V: Deserialize<'de>,
{
    type Value = RegistOrderMap<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = RegistOrderMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for RegistOrderMap<K, V>
where
    K: Deserialize<'de> + Eq,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(RegistOrderMapVisitor::new())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde")]
    use serde_json;

    #[test]
    fn test_insert() {
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let mut map = RegistOrderMap::new();
        map.insert(key2.clone(), 20);
        assert_eq!(map.get(&key1), None);
        assert_eq!(map.get(&key2), Some(&20));
        map.insert(key1.clone(), 10);
        assert_eq!(map.get(&key1), Some(&10));
        assert_eq!(map.get(&key2), Some(&20));
    }

    #[test]
    fn test_iter() {
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let mut map = RegistOrderMap::new();
        map.insert(key2.clone(), 20);
        map.insert(key1.clone(), 10);
        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&key2, &20)));
        assert_eq!(iter.next(), Some((&key1, &10)));
    }
    #[test]
    fn test_from() {
        let map = RegistOrderMap::from([
            ("key2", 20),
            ("key1", 10),
        ]);
        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&"key2", &20)));
        assert_eq!(iter.next(), Some((&"key1", &10)));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() {
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let mut map = RegistOrderMap::new();
        map.insert(key2.clone(), 20);
        map.insert(key1.clone(), 10);
        let json_str: &str = &serde_json::to_string(&map).unwrap();
        assert_eq!(json_str, r#"{"key2":20,"key1":10}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let json_str = r#"{"key2":20,"key1":10}"#;
        let map: RegistOrderMap<String, i64> = serde_json::from_str(json_str).unwrap();
        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&key2, &20)));
        assert_eq!(iter.next(), Some((&key1, &10)));
    }
}
