use afastdata::{AFastDeserialize, AFastSerialize, Error, ErrorKind, ValidateError};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn roundtrip<T: AFastSerialize + AFastDeserialize + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = val.to_bytes();
    let (decoded, offset) = T::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(*val, decoded);
}

// ==================== 整数类型 / Integer Types ====================

#[test]
fn test_i8() {
    roundtrip(&0i8);
    roundtrip(&42i8);
    roundtrip(&i8::MIN);
    roundtrip(&i8::MAX);
}

#[test]
fn test_u8() {
    roundtrip(&0u8);
    roundtrip(&255u8);
    roundtrip(&u8::MIN);
    roundtrip(&u8::MAX);
}

#[test]
fn test_i16() {
    roundtrip(&0i16);
    roundtrip(&-1000i16);
    roundtrip(&i16::MIN);
    roundtrip(&i16::MAX);
}

#[test]
fn test_u16() {
    roundtrip(&0u16);
    roundtrip(&60000u16);
    roundtrip(&u16::MIN);
    roundtrip(&u16::MAX);
}

#[test]
fn test_i32() {
    roundtrip(&0i32);
    roundtrip(&-100_000i32);
    roundtrip(&i32::MIN);
    roundtrip(&i32::MAX);
}

#[test]
fn test_u32() {
    roundtrip(&0u32);
    roundtrip(&3_000_000_000u32);
    roundtrip(&u32::MIN);
    roundtrip(&u32::MAX);
}

#[test]
fn test_i64() {
    roundtrip(&0i64);
    roundtrip(&-1_000_000_000_000i64);
    roundtrip(&i64::MIN);
    roundtrip(&i64::MAX);
}

#[test]
fn test_u64() {
    roundtrip(&0u64);
    roundtrip(&18_000_000_000_000_000_000u64);
    roundtrip(&u64::MIN);
    roundtrip(&u64::MAX);
}

#[test]
fn test_i128() {
    roundtrip(&0i128);
    roundtrip(&i128::MIN);
    roundtrip(&i128::MAX);
}

#[test]
fn test_u128() {
    roundtrip(&0u128);
    roundtrip(&u128::MIN);
    roundtrip(&u128::MAX);
}

#[test]
fn test_usize() {
    roundtrip(&0usize);
    roundtrip(&42usize);
    roundtrip(&usize::MAX);
}

// ==================== 浮点类型 / Float Types ====================

#[test]
fn test_f32() {
    roundtrip(&0.0f32);
    roundtrip(&std::f32::consts::PI);
    roundtrip(&f32::MIN);
    roundtrip(&f32::MAX);
}

#[test]
fn test_f64() {
    roundtrip(&0.0f64);
    roundtrip(&std::f64::consts::E);
    roundtrip(&f64::MIN);
    roundtrip(&f64::MAX);
}

// ==================== bool ====================

#[test]
fn test_bool() {
    roundtrip(&true);
    roundtrip(&false);
}

#[test]
fn test_bool_invalid_value() {
    let err = bool::from_bytes(&[2u8]).unwrap_err();
    assert!(format!("{}", err).contains("Invalid bool value"));
}

// ==================== String ====================

#[test]
fn test_string() {
    roundtrip(&String::from("hello world"));
    roundtrip(&String::from(""));
    roundtrip(&String::from("你好世界"));
}

#[test]
fn test_string_invalid_utf8() {
    let mut data = vec![3, 0, 0, 0];
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    let err = String::from_bytes(&data).unwrap_err();
    assert!(format!("{}", err).contains("Invalid UTF-8"));
}

// ==================== &str (仅序列化) ====================

#[test]
fn test_str_to_bytes() {
    let s: &str = "hello";
    let bytes = s.to_bytes();
    let (decoded, _) = String::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, "hello");
}

#[test]
fn test_str_to_bytes_with() {
    let s: &str = "hello";
    let bytes = s.to_bytes_with("marker");
    let bytes_normal = s.to_bytes();
    assert_eq!(bytes, bytes_normal);
}

// ==================== Vec<T> ====================

#[test]
fn test_vec() {
    roundtrip(&vec![1i32, 2, 3, 4, 5]);
    roundtrip(&Vec::<i32>::new());
    roundtrip(&vec![String::from("a"), String::from("b")]);
}

#[test]
fn test_vec_to_bytes_with() {
    let v = vec![1i32, 2, 3];
    let bytes = v.to_bytes_with("test");
    let bytes_normal = v.to_bytes();
    assert_eq!(bytes, bytes_normal);
    let (decoded, _) = Vec::<i32>::from_bytes_with(&bytes, "test").unwrap();
    assert_eq!(decoded, v);
}

#[test]
fn test_vec_truncated() {
    let data = vec![3u8, 0, 0, 0, 1, 0]; // len=3 但只有 2 字节
    let err = Vec::<u8>::from_bytes(&data).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

// ==================== Option<T> ====================

#[test]
fn test_option_some() {
    roundtrip(&Some(42i32));
    roundtrip(&Some(String::from("hello")));
}

#[test]
fn test_option_none() {
    roundtrip(&None::<i32>);
    roundtrip(&None::<String>);
}

#[test]
fn test_option_to_bytes_with_some() {
    let val = Some(42i32);
    let bytes = val.to_bytes_with("marker");
    let bytes_normal = val.to_bytes();
    assert_eq!(bytes, bytes_normal);
}

#[test]
fn test_option_to_bytes_with_none() {
    let val = None::<i32>;
    let bytes = val.to_bytes_with("marker");
    assert_eq!(bytes, vec![0u8]);
}

#[test]
fn test_option_from_bytes_with_some() {
    let bytes = vec![1u8, 42, 0, 0, 0];
    let (val, _) = Option::<i32>::from_bytes_with(&bytes, "marker").unwrap();
    assert_eq!(val, Some(42));
}

#[test]
fn test_option_from_bytes_with_none() {
    let bytes = vec![0u8];
    let (val, _) = Option::<i32>::from_bytes_with(&bytes, "marker").unwrap();
    assert_eq!(val, None);
}

#[test]
fn test_option_invalid_tag() {
    let err = Option::<i32>::from_bytes(&[2u8]).unwrap_err();
    assert!(format!("{}", err).contains("Invalid Option tag"));
}

#[test]
fn test_option_from_bytes_with_invalid_tag() {
    let err = Option::<i32>::from_bytes_with(&[2u8], "marker").unwrap_err();
    assert!(format!("{}", err).contains("Invalid Option tag"));
}

#[test]
fn test_option_truncated() {
    let err = Option::<i32>::from_bytes(&[]).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

// ==================== [T; N] ====================

#[test]
fn test_array() {
    roundtrip(&[1i32, 2, 3]);
    roundtrip(&[0u8; 0]);
    roundtrip(&[42u8]);
}

#[test]
fn test_array_to_bytes_with() {
    let arr = [1i32, 2, 3];
    let bytes = arr.to_bytes_with("marker");
    let bytes_normal = arr.to_bytes();
    assert_eq!(bytes, bytes_normal);
}

#[test]
fn test_array_from_bytes_with() {
    let arr = [1i32, 2, 3];
    let bytes = arr.to_bytes();
    let (decoded, _) = <[i32; 3]>::from_bytes_with(&bytes, "marker").unwrap();
    assert_eq!(decoded, arr);
}

// ==================== HashMap ====================

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
fn test_hashmap_with() {
    let mut map = HashMap::new();
    map.insert(1i32, String::from("one"));
    map.insert(2, String::from("two"));
    let bytes = map.to_bytes_with("test");
    let (decoded, offset) = HashMap::<i32, String>::from_bytes_with(&bytes, "test").unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(map, decoded);
}

// ==================== HashSet ====================

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
fn test_hashset_with() {
    let mut set = HashSet::new();
    set.insert(10i32);
    set.insert(20);
    let bytes = set.to_bytes_with("test");
    let (decoded, offset) = HashSet::<i32>::from_bytes_with(&bytes, "test").unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(set, decoded);
}

// ==================== BTreeMap ====================

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
fn test_btreemap_with() {
    let mut map = BTreeMap::new();
    map.insert(String::from("a"), 1i32);
    map.insert(String::from("b"), 2);
    let bytes = map.to_bytes_with("test");
    let (decoded, offset) = BTreeMap::<String, i32>::from_bytes_with(&bytes, "test").unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(map, decoded);
}

// ==================== BTreeSet ====================

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
fn test_btreeset_with() {
    let mut set = BTreeSet::new();
    set.insert(100u64);
    set.insert(200);
    let bytes = set.to_bytes_with("test");
    let (decoded, offset) = BTreeSet::<u64>::from_bytes_with(&bytes, "test").unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(set, decoded);
}

// ==================== 元组 / Tuples ====================

#[test]
fn test_tuple_2() {
    roundtrip(&(1i32, 2.0f64));
}

#[test]
fn test_tuple_3() {
    roundtrip(&(1i32, String::from("hello"), true));
}

#[test]
fn test_tuple_4() {
    roundtrip(&(1u8, 2u16, 3u32, 4u64));
}

#[test]
fn test_tuple_1() {
    roundtrip(&(42i32,));
}

#[test]
fn test_tuple_to_bytes_with() {
    let t = (1i32, String::from("hello"));
    let bytes = t.to_bytes_with("marker");
    let bytes_normal = t.to_bytes();
    assert_eq!(bytes, bytes_normal);
}

#[test]
fn test_tuple_from_bytes_with() {
    let t = (1i32, String::from("hello"));
    let bytes = t.to_bytes();
    let (decoded, _) = <(i32, String)>::from_bytes_with(&bytes, "marker").unwrap();
    assert_eq!(decoded, t);
}

// ==================== Box<T> ====================

#[test]
fn test_box() {
    let val = Box::new(42i32);
    let bytes = val.to_bytes();
    let (decoded, offset) = Box::<i32>::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(*decoded, 42);
}

#[test]
fn test_box_to_bytes_with() {
    let val = Box::new(42i32);
    let bytes = val.to_bytes_with("marker");
    let bytes_normal = val.to_bytes();
    assert_eq!(bytes, bytes_normal);
}

#[test]
fn test_box_from_bytes_with() {
    let val = Box::new(42i32);
    let bytes = val.to_bytes();
    let (decoded, _) = Box::<i32>::from_bytes_with(&bytes, "marker").unwrap();
    assert_eq!(*decoded, 42);
}

#[test]
fn test_box_string() {
    let val = Box::new(String::from("boxed"));
    roundtrip(&val);
}

// ==================== Error 类型 / Error Types ====================

#[test]
fn test_error_serialize_display() {
    let err = Error::serialize("test error".to_string());
    assert_eq!(format!("{}", err), "Serialize error: test error");
}

#[test]
fn test_error_deserialize_display() {
    let err = Error::deserialize("bad data".to_string());
    assert_eq!(format!("{}", err), "Deserialize error: bad data");
}

#[test]
fn test_error_validate_display() {
    let err = Error::validate(42, "invalid".to_string());
    assert_eq!(
        format!("{}", err),
        "Validation error: code 42 is invalid: invalid"
    );
}

#[test]
fn test_error_serialize_debug() {
    let err = Error::serialize("test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Serialize error"));
}

#[test]
fn test_error_deserialize_debug() {
    let err = Error::deserialize("test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Deserialize error"));
}

#[test]
fn test_error_validate_debug() {
    let err = Error::validate(1, "msg".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Validation error"));
}

#[test]
fn test_error_kind_serialize() {
    let err = Error::serialize("msg".to_string());
    assert!(matches!(err.kind(), ErrorKind::SerializeError(_)));
}

#[test]
fn test_error_kind_deserialize() {
    let err = Error::deserialize("msg".to_string());
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

#[test]
fn test_error_kind_validate() {
    let err = Error::validate(99, "msg".to_string());
    assert!(matches!(err.kind(), ErrorKind::ValidateError(99, _)));
}

#[test]
fn test_errorkind_message_serialize() {
    let kind = ErrorKind::SerializeError("s msg".to_string());
    assert_eq!(kind.message(), "s msg");
}

#[test]
fn test_errorkind_message_deserialize() {
    let kind = ErrorKind::DeserializeError("d msg".to_string());
    assert_eq!(kind.message(), "d msg");
}

#[test]
fn test_errorkind_message_validate() {
    let kind = ErrorKind::ValidateError(1, "v msg".to_string());
    assert_eq!(kind.message(), "v msg");
}

#[test]
fn test_errorkind_code_validate() {
    let kind = ErrorKind::ValidateError(42, "msg".to_string());
    assert_eq!(kind.code(), Some(42));
}

#[test]
fn test_errorkind_code_serialize() {
    let kind = ErrorKind::SerializeError("msg".to_string());
    assert_eq!(kind.code(), None);
}

#[test]
fn test_errorkind_code_deserialize() {
    let kind = ErrorKind::DeserializeError("msg".to_string());
    assert_eq!(kind.code(), None);
}

#[test]
fn test_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(Error::serialize("test".to_string()));
    assert!(err.to_string().contains("Serialize error"));
}

#[test]
fn test_validate_error_new() {
    let ve = ValidateError::new(10, "bad value".to_string());
    assert_eq!(
        format!("{}", ve),
        "Validation error: code 10, message bad value"
    );
}

#[test]
fn test_validate_error_display() {
    let ve = ValidateError::new(5, "err".to_string());
    assert_eq!(format!("{}", ve), "Validation error: code 5, message err");
}

#[test]
fn test_validate_error_debug() {
    let ve = ValidateError::new(5, "err".to_string());
    let debug = format!("{:?}", ve);
    assert!(debug.contains("Validation error"));
    assert!(debug.contains("code 5"));
}

#[test]
fn test_validate_error_to_afastdata_error() {
    let ve = ValidateError::new(7, "custom".to_string());
    let err = ve.to_afastdata_error();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(7, _)));
}

#[test]
fn test_validate_error_is_std_error() {
    let ve = ValidateError::new(1, "msg".to_string());
    let err: Box<dyn std::error::Error> = Box::new(ve);
    assert!(err.to_string().contains("Validation error"));
}

// ==================== read_exact 错误路径 / read_exact Error Paths ====================

#[test]
fn test_read_exact_insufficient_bytes() {
    let err = i32::from_bytes(&[]).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

#[test]
fn test_read_exact_partial() {
    let err = i32::from_bytes(&[1, 0]).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

#[test]
fn test_string_truncated_len() {
    let data = vec![100, 0, 0, 0, 1, 2, 3];
    let err = String::from_bytes(&data).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

#[test]
fn test_vec_truncated_len_prefix() {
    let err = Vec::<u8>::from_bytes(&[]).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DeserializeError(_)));
}

// ==================== LEN_INT_SIZE ====================

#[test]
fn test_len_int_size() {
    assert_eq!(afastdata::LEN_INT_SIZE, std::mem::size_of::<u32>());
}
