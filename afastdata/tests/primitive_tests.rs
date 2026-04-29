use afastdata::{AFastDeserialize, AFastSerialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn roundtrip<T: AFastSerialize + AFastDeserialize + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = val.to_bytes();
    let (decoded, offset) = T::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(*val, decoded);
}

#[test]
fn test_integers() {
    roundtrip(&42i8);
    roundtrip(&255u8);
    roundtrip(&-1000i16);
    roundtrip(&60000u16);
    roundtrip(&-100_000i32);
    roundtrip(&3_000_000_000u32);
    roundtrip(&-1_000_000_000_000i64);
    roundtrip(&18_000_000_000_000_000_000u64);
}

#[test]
fn test_floats() {
    roundtrip(&3.14f32);
    roundtrip(&2.718281828f64);
}

#[test]
fn test_bool() {
    roundtrip(&true);
    roundtrip(&false);
}

#[test]
fn test_string() {
    roundtrip(&String::from("hello world"));
    roundtrip(&String::from(""));
    roundtrip(&String::from("你好"));
}

#[test]
fn test_vec() {
    roundtrip(&vec![1i32, 2, 3, 4, 5]);
    roundtrip(&Vec::<i32>::new());
    roundtrip(&vec![String::from("a"), String::from("b")]);
}

#[test]
fn test_option() {
    roundtrip(&Some(42i32));
    roundtrip(&None::<i32>);
    roundtrip(&Some(String::from("hello")));
}

#[test]
fn test_array() {
    roundtrip(&[1i32, 2, 3]);
}

#[test]
fn test_hashmap() {
    let mut map = HashMap::new();
    map.insert(1i32, String::from("one"));
    map.insert(2, String::from("two"));
    let bytes = map.to_bytes();
    let (decoded, offset) = HashMap::<i32, String>::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(map, decoded);
}

#[test]
fn test_hashset() {
    let mut set = HashSet::new();
    set.insert(10i32);
    set.insert(20);
    set.insert(30);
    let bytes = set.to_bytes();
    let (decoded, offset) = HashSet::<i32>::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(set, decoded);
}

#[test]
fn test_btreemap() {
    let mut map = BTreeMap::new();
    map.insert(String::from("a"), 1i32);
    map.insert(String::from("b"), 2);
    let bytes = map.to_bytes();
    let (decoded, offset) = BTreeMap::<String, i32>::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(map, decoded);
}

#[test]
fn test_btreeset() {
    let mut set = BTreeSet::new();
    set.insert(100u64);
    set.insert(200);
    let bytes = set.to_bytes();
    let (decoded, offset) = BTreeSet::<u64>::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(set, decoded);
}

#[test]
fn test_tuples() {
    roundtrip(&(1i32, 2.0f64));
    roundtrip(&(1i32, String::from("hello"), true));
    roundtrip(&(1u8, 2u16, 3u32, 4u64));
    roundtrip(&(42i32,));
}
