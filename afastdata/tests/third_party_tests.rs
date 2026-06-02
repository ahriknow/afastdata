#![cfg(any(
    feature = "uuid",
    feature = "chrono",
    feature = "rust_decimal",
    feature = "url",
    feature = "bytes",
    feature = "bigdecimal",
    feature = "ipnetwork",
    feature = "mac_address",
    feature = "sqlx"
))]

//! 第三方类型序列化/反序列化测试
//!
//! Tests for third-party type serialization/deserialization

use afastdata::{AFastDeserialize, AFastSerialize};

// ==================== std::net ====================

#[test]
fn test_ipv4_addr() {
    use std::net::Ipv4Addr;
    let addr = Ipv4Addr::new(192, 168, 1, 1);
    let bytes = addr.to_bytes();
    assert_eq!(bytes, vec![192, 168, 1, 1]);
    let (decoded, consumed) = Ipv4Addr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, 4);
}

#[test]
fn test_ipv4_loopback() {
    use std::net::Ipv4Addr;
    let addr = Ipv4Addr::LOCALHOST;
    let bytes = addr.to_bytes();
    let (decoded, consumed) = Ipv4Addr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, 4);
}

#[test]
fn test_ipv6_addr() {
    use std::net::Ipv6Addr;
    let addr = Ipv6Addr::LOCALHOST;
    let bytes = addr.to_bytes();
    let (decoded, consumed) = Ipv6Addr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, 16);
}

#[test]
fn test_ipv6_from_bytes() {
    use std::net::Ipv6Addr;
    let addr = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
    let bytes = addr.to_bytes();
    let (decoded, consumed) = Ipv6Addr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, 16);
}

#[test]
fn test_ip_addr_v4() {
    use std::net::{IpAddr, Ipv4Addr};
    let addr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let bytes = addr.to_bytes();
    assert_eq!(bytes[0], 0); // V4 tag
    let (decoded, consumed) = IpAddr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_ip_addr_v6() {
    use std::net::{IpAddr, Ipv6Addr};
    let addr = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let bytes = addr.to_bytes();
    assert_eq!(bytes[0], 1); // V6 tag
    let (decoded, consumed) = IpAddr::from_bytes(&bytes).unwrap();
    assert_eq!(addr, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_ip_addr_invalid_tag() {
    use std::net::IpAddr;
    let bytes = vec![5u8];
    let result = IpAddr::from_bytes(&bytes);
    assert!(result.is_err());
}

// ==================== uuid ====================

#[cfg(feature = "uuid")]
#[test]
fn test_uuid_roundtrip() {
    let id = uuid::Uuid::from_fields(
        0x12345678,
        0x1234,
        0x5678,
        &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11],
    );
    let bytes = id.to_bytes();
    assert_eq!(bytes.len(), 16);
    let (decoded, consumed) = <uuid::Uuid as AFastDeserialize>::from_bytes(&bytes).unwrap();
    assert_eq!(id, decoded);
    assert_eq!(consumed, 16);
}

#[cfg(feature = "uuid")]
#[test]
fn test_uuid_nil() {
    let id = uuid::Uuid::nil();
    let bytes = id.to_bytes();
    assert_eq!(bytes, vec![0u8; 16]);
    let (decoded, consumed) = <uuid::Uuid as AFastDeserialize>::from_bytes(&bytes).unwrap();
    assert_eq!(id, decoded);
    assert_eq!(consumed, 16);
}

// ==================== chrono ====================

#[cfg(feature = "chrono")]
#[test]
fn test_naive_date_roundtrip() {
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    let bytes = date.to_bytes();
    assert_eq!(bytes.len(), 6);
    let (decoded, consumed) = NaiveDate::from_bytes(&bytes).unwrap();
    assert_eq!(date, decoded);
    assert_eq!(consumed, 6);
}

#[cfg(feature = "chrono")]
#[test]
fn test_naive_time_roundtrip() {
    use chrono::NaiveTime;
    let time = NaiveTime::from_hms_nano_opt(14, 30, 45, 123456789).unwrap();
    let bytes = time.to_bytes();
    assert_eq!(bytes.len(), 8);
    let (decoded, consumed) = NaiveTime::from_bytes(&bytes).unwrap();
    assert_eq!(time, decoded);
    assert_eq!(consumed, 8);
}

#[cfg(feature = "chrono")]
#[test]
fn test_naive_datetime_roundtrip() {
    use chrono::{NaiveDate, NaiveTime};
    let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    let time = NaiveTime::from_hms_nano_opt(14, 30, 45, 123456789).unwrap();
    let dt = date.and_time(time);
    let bytes = dt.to_bytes();
    assert_eq!(bytes.len(), 14);
    let (decoded, consumed) = chrono::NaiveDateTime::from_bytes(&bytes).unwrap();
    assert_eq!(dt, decoded);
    assert_eq!(consumed, 14);
}

#[cfg(feature = "chrono")]
#[test]
fn test_datetime_utc_roundtrip() {
    use chrono::DateTime;
    let dt = DateTime::from_timestamp(1700000000, 500000000).unwrap();
    let bytes = dt.to_bytes();
    assert_eq!(bytes.len(), 12);
    let (decoded, consumed) = DateTime::<chrono::Utc>::from_bytes(&bytes).unwrap();
    assert_eq!(dt, decoded);
    assert_eq!(consumed, 12);
}

#[cfg(feature = "chrono")]
#[test]
fn test_datetime_utc_epoch() {
    use chrono::DateTime;
    let dt = DateTime::from_timestamp(0, 0).unwrap();
    let bytes = dt.to_bytes();
    let (decoded, consumed) = DateTime::<chrono::Utc>::from_bytes(&bytes).unwrap();
    assert_eq!(dt, decoded);
    assert_eq!(consumed, 12);
}

#[cfg(feature = "chrono")]
#[test]
fn test_datetime_local_roundtrip() {
    use chrono::DateTime;
    let dt_utc = DateTime::from_timestamp(1700000000, 0).unwrap();
    let dt_local = dt_utc.with_timezone(&chrono::Local);
    let bytes = dt_local.to_bytes();
    assert_eq!(bytes.len(), 12);
    let (decoded, consumed) = DateTime::<chrono::Local>::from_bytes(&bytes).unwrap();
    assert_eq!(dt_local, decoded);
    assert_eq!(consumed, 12);
}

#[cfg(feature = "chrono")]
#[test]
fn test_naive_date_invalid() {
    use chrono::NaiveDate;
    // month = 13 is invalid
    let bytes = vec![0xe9, 0x07, 0x00, 0x00, 13, 1];
    let result = NaiveDate::from_bytes(&bytes);
    assert!(result.is_err());
}

// ==================== rust_decimal ====================

#[cfg(feature = "rust_decimal")]
#[test]
fn test_decimal_roundtrip() {
    use rust_decimal::Decimal;
    let d = Decimal::new(123456789, 4); // 12345.6789
    let bytes = d.to_bytes();
    assert_eq!(bytes.len(), 20);
    let (decoded, consumed) = Decimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, 20);
}

#[cfg(feature = "rust_decimal")]
#[test]
fn test_decimal_zero() {
    use rust_decimal::Decimal;
    let d = Decimal::ZERO;
    let bytes = d.to_bytes();
    let (decoded, consumed) = Decimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, 20);
}

#[cfg(feature = "rust_decimal")]
#[test]
fn test_decimal_negative() {
    use rust_decimal::Decimal;
    let d = Decimal::new(-99999, 2); // -999.99
    let bytes = d.to_bytes();
    let (decoded, consumed) = Decimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, 20);
}

// ==================== url ====================

#[cfg(feature = "url")]
#[test]
fn test_url_roundtrip() {
    let url = url::Url::parse("https://example.com/path?query=1&foo=bar#frag").unwrap();
    let bytes = url.to_bytes();
    let (decoded, consumed) = url::Url::from_bytes(&bytes).unwrap();
    assert_eq!(url, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "url")]
#[test]
fn test_url_with_unicode() {
    let url = url::Url::parse("https://例子.测试/路径").unwrap();
    let bytes = url.to_bytes();
    let (decoded, consumed) = url::Url::from_bytes(&bytes).unwrap();
    assert_eq!(url.as_str(), decoded.as_str());
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "url")]
#[test]
fn test_url_invalid() {
    let data = b"not a valid url";
    let mut bytes = Vec::new();
    bytes.extend((data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    let result = url::Url::from_bytes(&bytes);
    assert!(result.is_err());
}

// ==================== bytes ====================

#[cfg(feature = "bytes")]
#[test]
fn test_bytes_roundtrip() {
    let b = bytes::Bytes::from(vec![1u8, 2, 3, 4, 5]);
    let bytes = b.to_bytes();
    let (decoded, consumed) = bytes::Bytes::from_bytes(&bytes).unwrap();
    assert_eq!(b, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bytes")]
#[test]
fn test_bytes_empty() {
    let b = bytes::Bytes::new();
    let bytes = b.to_bytes();
    let (decoded, consumed) = bytes::Bytes::from_bytes(&bytes).unwrap();
    assert_eq!(b, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bytes")]
#[test]
fn test_bytes_mut_roundtrip() {
    let b = bytes::BytesMut::from(&b"hello world"[..]);
    let bytes = b.to_bytes();
    let (decoded, consumed) = bytes::BytesMut::from_bytes(&bytes).unwrap();
    assert_eq!(b, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bytes")]
#[test]
fn test_bytes_mut_empty() {
    let b = bytes::BytesMut::new();
    let bytes = b.to_bytes();
    let (decoded, consumed) = bytes::BytesMut::from_bytes(&bytes).unwrap();
    assert_eq!(b, decoded);
    assert_eq!(consumed, bytes.len());
}

// ==================== bigdecimal ====================

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_roundtrip() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let d = BigDecimal::from_str("12345.6789").unwrap();
    let bytes = d.to_bytes();
    let (decoded, consumed) = BigDecimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_zero() {
    use bigdecimal::BigDecimal;
    let d = BigDecimal::from(0);
    let bytes = d.to_bytes();
    let (decoded, consumed) = BigDecimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_negative() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let d = BigDecimal::from_str("-999.99").unwrap();
    let bytes = d.to_bytes();
    let (decoded, consumed) = BigDecimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_large() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let d = BigDecimal::from_str("123456789012345678901234567890.123456789").unwrap();
    let bytes = d.to_bytes();
    let (decoded, consumed) = BigDecimal::from_bytes(&bytes).unwrap();
    assert_eq!(d, decoded);
    assert_eq!(consumed, bytes.len());
}

// ==================== ipnetwork ====================

#[cfg(feature = "ipnetwork")]
#[test]
fn test_ipv4_network_roundtrip() {
    use ipnetwork::Ipv4Network;
    use std::net::Ipv4Addr;
    let net = Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    let bytes = net.to_bytes();
    assert_eq!(bytes.len(), 5);
    let (decoded, consumed) = Ipv4Network::from_bytes(&bytes).unwrap();
    assert_eq!(net, decoded);
    assert_eq!(consumed, 5);
}

#[cfg(feature = "ipnetwork")]
#[test]
fn test_ipv6_network_roundtrip() {
    use ipnetwork::Ipv6Network;
    use std::net::Ipv6Addr;
    let net = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    let bytes = net.to_bytes();
    assert_eq!(bytes.len(), 17);
    let (decoded, consumed) = Ipv6Network::from_bytes(&bytes).unwrap();
    assert_eq!(net, decoded);
    assert_eq!(consumed, 17);
}

#[cfg(feature = "ipnetwork")]
#[test]
fn test_ip_network_v4() {
    use ipnetwork::IpNetwork;
    use std::net::Ipv4Addr;
    let net = IpNetwork::V4(ipnetwork::Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap());
    let bytes = net.to_bytes();
    assert_eq!(bytes[0], 0); // V4 tag
    let (decoded, consumed) = IpNetwork::from_bytes(&bytes).unwrap();
    assert_eq!(net, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "ipnetwork")]
#[test]
fn test_ip_network_v6() {
    use ipnetwork::IpNetwork;
    use std::net::Ipv6Addr;
    let net = IpNetwork::V6(ipnetwork::Ipv6Network::new(Ipv6Addr::LOCALHOST, 128).unwrap());
    let bytes = net.to_bytes();
    assert_eq!(bytes[0], 1); // V6 tag
    let (decoded, consumed) = IpNetwork::from_bytes(&bytes).unwrap();
    assert_eq!(net, decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "ipnetwork")]
#[test]
fn test_ip_network_invalid_tag() {
    use ipnetwork::IpNetwork;
    let bytes = vec![5u8];
    let result = IpNetwork::from_bytes(&bytes);
    assert!(result.is_err());
}

// ==================== mac_address ====================

#[cfg(feature = "mac_address")]
#[test]
fn test_mac_address_roundtrip() {
    let mac = mac_address::MacAddress::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    let bytes = mac.to_bytes();
    assert_eq!(bytes, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    let (decoded, consumed) = mac_address::MacAddress::from_bytes(&bytes).unwrap();
    assert_eq!(mac, decoded);
    assert_eq!(consumed, 6);
}

#[cfg(feature = "mac_address")]
#[test]
fn test_mac_address_zero() {
    let mac = mac_address::MacAddress::new([0; 6]);
    let bytes = mac.to_bytes();
    assert_eq!(bytes, vec![0u8; 6]);
    let (decoded, consumed) = mac_address::MacAddress::from_bytes(&bytes).unwrap();
    assert_eq!(mac, decoded);
    assert_eq!(consumed, 6);
}

// ==================== sqlx::Json<T> ====================

#[cfg(feature = "sqlx")]
#[test]
fn test_sqlx_json_string() {
    let val = sqlx::types::Json(String::from("hello"));
    let bytes = val.to_bytes();
    let (decoded, consumed) =
        <sqlx::types::Json<String> as AFastDeserialize>::from_bytes(&bytes).unwrap();
    assert_eq!(*val, *decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "sqlx")]
#[test]
fn test_sqlx_json_vec() {
    let val = sqlx::types::Json(vec![1i32, 2, 3]);
    let bytes = val.to_bytes();
    let (decoded, consumed) =
        <sqlx::types::Json<Vec<i32>> as AFastDeserialize>::from_bytes(&bytes).unwrap();
    assert_eq!(*val, *decoded);
    assert_eq!(consumed, bytes.len());
}

#[cfg(feature = "sqlx")]
#[test]
fn test_sqlx_json_nested() {
    let val = sqlx::types::Json(Some(vec![String::from("a"), String::from("b")]));
    let bytes = val.to_bytes();
    let (decoded, consumed) =
        <sqlx::types::Json<Option<Vec<String>>> as AFastDeserialize>::from_bytes(&bytes).unwrap();
    assert_eq!(*val, *decoded);
    assert_eq!(consumed, bytes.len());
}
