#![cfg(feature = "serde_json")]

//! serde_json::Value 序列化/反序列化测试
//!
//! Tests for serde_json::Value serialization/deserialization

use afastdata::{AFastDeserialize, AFastSerialize};

#[test]
fn test_serde_json_null() {
    let value = serde_json::Value::Null;
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_bool() {
    let value = serde_json::Value::Bool(true);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());

    let value = serde_json::Value::Bool(false);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_number_i64() {
    let value = serde_json::json!(42i64);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());

    let value = serde_json::json!(-100i64);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_number_u64() {
    let value = serde_json::json!(u64::MAX);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_number_f64() {
    let value = serde_json::json!(3.14f64);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_string() {
    let value = serde_json::json!("hello world");
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_string_unicode() {
    let value = serde_json::json!("你好世界 🦀");
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_array() {
    let value = serde_json::json!([1, 2, 3, "four", true, null]);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_object() {
    let value = serde_json::json!({
        "name": "Alice",
        "age": 30,
        "active": true,
        "scores": [95, 87, 92]
    });
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_nested() {
    let value = serde_json::json!({
        "users": [
            {"name": "Alice", "metadata": {"age": 30}},
            {"name": "Bob", "metadata": {"age": 25}}
        ],
        "count": 2,
        "empty": null
    });
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_empty_array() {
    let value = serde_json::json!([]);
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_empty_object() {
    let value = serde_json::json!({});
    let bytes = value.to_bytes();
    let (decoded, consumed) = serde_json::Value::from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_serde_json_invalid_tag() {
    let bytes = vec![255u8]; // Invalid tag
    let result = serde_json::Value::from_bytes(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_serde_json_truncated() {
    let bytes = vec![3u8]; // String tag but no data
    let result = serde_json::Value::from_bytes(&bytes);
    assert!(result.is_err());
}
