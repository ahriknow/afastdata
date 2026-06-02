//! # afastdata
//!
//! 高性能二进制序列化/反序列化库，通过 derive 宏为 Rust 类型自动生成序列化代码。
//!
//! A high-performance binary serialization/deserialization library that automatically
//! generates serialization code for Rust types via derive macros.
//!
//! ## 编码规则 / Encoding Rules
//!
//! | 类型 / Type | 编码方式 / Encoding |
//! |---|---|
//! | `i8`, `u8` | 1 字节 little-endian / 1 byte little-endian |
//! | `i16`, `u16` | 2 字节 little-endian / 2 bytes little-endian |
//! | `i32`, `u32` | 4 字节 little-endian / 4 bytes little-endian |
//! | `i64`, `u64` | 8 字节 little-endian / 8 bytes little-endian |
//! | `usize` | u64 little-endian（跨平台兼容）/ u64 little-endian (cross-platform) |
//! | `i128`, `u128` | 16 字节 little-endian / 16 bytes little-endian |
//! | `f32` | 4 字节 IEEE 754 / 4 bytes IEEE 754 |
//! | `f64` | 8 字节 IEEE 754 / 8 bytes IEEE 754 |
//! | `bool` | 1 字节，`0x00`=false，`0x01`=true / 1 byte, `0x00`=false, `0x01`=true |
//! | `String` | 长度前缀 (LenInt) + UTF-8 字节 / Length prefix (LenInt) + UTF-8 bytes |
//! | `Vec<T>` | 长度前缀 (LenInt) + 逐元素编码 / Length prefix (LenInt) + element-wise encoding |
//! | `Option<T>` | 标记字节 (`0x00`=None, `0x01`=Some) + 数据（仅 Some 时）/ Tag byte (`0x00`=None, `0x01`=Some) + data (only when Some) |
//! | `[T; N]` | 逐元素编码，无长度前缀 / Element-wise encoding, no length prefix |
//! | `&str` | 长度前缀 (LenInt) + UTF-8 字节（仅序列化）/ Length prefix (LenInt) + UTF-8 bytes (serialize only) |
//!
//! ## 长度前缀类型 / Length Prefix Type
//!
//! 默认使用 `u32` 作为长度前缀（最大 4GB），可通过启用 `len-u64` feature 切换为 `u64`。
//!
//! By default, `u32` is used as the length prefix (max 4GB). Enable the `len-u64`
//! feature to switch to `u64`.

mod error;

pub use error::{Error, ErrorKind, ValidateError};

/// Re-export derive 宏。
///
/// Re-exports derive macros from [`afastdata_macro`].
pub use afastdata_macro::*;

/// 长度前缀使用的整数类型。默认为 `u32`，启用 `len-u64` feature 后为 `u64`。
///
/// The integer type used for length prefixes. Defaults to `u32`, switches to `u64`
/// when the `len-u64` feature is enabled.
#[cfg(feature = "len-u64")]
pub type LenInt = u64;

/// 长度前缀使用的整数类型。默认为 `u32`，启用 `len-u64` feature 后为 `u64`。
///
/// The integer type used for length prefixes. Defaults to `u32`, switches to `u64`
/// when the `len-u64` feature is enabled.
#[cfg(not(feature = "len-u64"))]
pub type LenInt = u32;

/// 长度前缀类型的字节大小（`u32` 为 4，`u64` 为 8）。
///
/// The byte size of the length prefix type (`4` for `u32`, `8` for `u64`).
pub const LEN_INT_SIZE: usize = std::mem::size_of::<LenInt>();

/// 序列化 trait，为类型提供转换为字节数组的能力。
///
/// Serialization trait that provides the ability to convert a type into a byte array.
///
/// # 编码规则 / Encoding Rules
///
/// 实现者应确保 `to_bytes()` 输出的字节流能够被对应的 [`AFastDeserialize::from_bytes`]
/// 完整还原。字节序统一使用 **小端序 (little-endian)**。
///
/// Implementors must ensure that the byte output of `to_bytes()` can be fully
/// restored by the corresponding [`AFastDeserialize::from_bytes`]. All multi-byte
/// values use **little-endian** byte order.
///
/// # 示例 / Example
///
/// ```
/// use afastdata::AFastSerialize;
///
/// let value: i32 = 42;
/// let bytes = value.to_bytes();
/// assert_eq!(bytes, vec![42, 0, 0, 0]);
/// ```
pub trait AFastSerialize {
    /// 将值序列化为字节数组。
    ///
    /// Serialize the value into a byte array.
    ///
    /// # 返回值 / Returns
    ///
    /// 返回一个 `Vec<u8>`，包含该值的完整二进制表示。
    ///
    /// Returns a `Vec<u8>` containing the complete binary representation of the value.
    fn to_bytes(&self) -> Vec<u8>;

    /// 将值序列化为字节数组，跳过与指定 marker 匹配的 `skip_with` 字段。
    ///
    /// Serialize the value into a byte array, skipping `skip_with` fields whose
    /// marker matches the given marker.
    ///
    /// 默认实现直接调用 `to_bytes()`（即不跳过任何字段）。
    ///
    /// The default implementation simply calls `to_bytes()` (i.e., skips no fields).
    fn to_bytes_with(&self, _marker: &str) -> Vec<u8> {
        self.to_bytes()
    }
}

/// 反序列化 trait，为类型提供从字节数组还原的能力。
///
/// Deserialization trait that provides the ability to restore a type from a byte array.
///
/// # 返回值说明 / Return Value Notes
///
/// `from_bytes` 返回 `Result<(Self, usize), Error>`：
/// - `Ok((value, bytes_consumed))`：成功时返回还原的值和实际消耗的字节数
/// - `Err(message)`：失败时返回错误描述
///
/// `from_bytes` returns `Result<(Self, usize), Error>`:
/// - `Ok((value, bytes_consumed))`: On success, returns the restored value and
///   the number of bytes actually consumed
/// - `Err(message)`: On failure, returns an error description
///
/// # 示例 / Example
///
/// ```
/// use afastdata::AFastDeserialize;
///
/// let bytes: Vec<u8> = vec![42, 0, 0, 0];
/// let (value, consumed) = i32::from_bytes(&bytes).unwrap();
/// assert_eq!(value, 42);
/// assert_eq!(consumed, 4);
/// ```
pub trait AFastDeserialize: Sized {
    /// 从字节数组中反序列化一个值。
    ///
    /// Deserialize a value from a byte array.
    ///
    /// # 参数 / Parameters
    ///
    /// - `data`：待反序列化的字节切片，可以是完整数据的子集（从指定偏移量开始）
    ///
    /// - `data`: The byte slice to deserialize from. May be a subset of the complete
    ///   data (starting from a specific offset).
    ///
    /// # 错误 / Errors
    ///
    /// 当字节数据不足或格式无效时返回 `Err`。
    ///
    /// Returns `Err` when there are insufficient bytes or the format is invalid.
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error>;

    /// 从字节数组中反序列化一个值，跳过与指定 marker 匹配的 `skip_with` 字段。
    ///
    /// Deserialize a value from a byte array, skipping `skip_with` fields whose
    /// marker matches the given marker (those fields use default values or custom
    /// functions instead of reading from the byte stream).
    ///
    /// 默认实现直接调用 `from_bytes()`（即不跳过任何字段）。
    ///
    /// The default implementation simply calls `from_bytes()` (i.e., skips no fields).
    fn from_bytes_with(data: &[u8], _marker: &str) -> Result<(Self, usize), Error> {
        Self::from_bytes(data)
    }
}

/// 从字节切片中精确读取指定数量的字节。内部辅助函数。
///
/// Reads exactly `n` bytes from a byte slice at the given offset. Internal helper.
///
/// # 参数 / Parameters
///
/// - `data`：源字节切片 / Source byte slice
/// - `offset`：起始偏移量 / Starting offset
/// - `n`：需要读取的字节数 / Number of bytes to read
///
/// # 错误 / Errors
///
/// 当 `offset + n` 超出 `data` 长度时返回错误。
///
/// Returns an error when `offset + n` exceeds the length of `data`.
fn read_exact(data: &[u8], offset: usize, n: usize) -> Result<&[u8], Error> {
    if offset + n > data.len() {
        Err(Error::deserialize(format!(
            "Not enough bytes: need {} at offset {}, have {}",
            n,
            offset,
            data.len()
        )))
    } else {
        Ok(&data[offset..offset + n])
    }
}

// ==================== 整数和浮点类型 / Integer and Float Types ====================

/// 为整数和浮点类型实现序列化/反序列化的宏。
///
/// Macro to implement serialization/deserialization for integer and float types.
///
/// 每种类型使用其原生字节大小，采用 little-endian 字节序。
///
/// Each type uses its native byte size with little-endian byte order.
macro_rules! impl_serialize_int {
    ($t:ty, $size:expr) => {
        impl AFastSerialize for $t {
            /// 将数值转为 `$size` 字节的 little-endian 字节数组。
            ///
            /// Converts the value to a `$size`-byte little-endian byte array.
            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }
        }
        impl AFastDeserialize for $t {
            /// 从字节数组中读取 `$size` 字节并还原为数值。
            ///
            /// Reads `$size` bytes from the byte array and restores the value.
            fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
                let bytes = read_exact(data, 0, $size)?;
                let arr: [u8; $size] = bytes.try_into().unwrap();
                Ok((Self::from_le_bytes(arr), $size))
            }
        }
    };
}

impl_serialize_int!(i8, 1);
impl_serialize_int!(u8, 1);
impl_serialize_int!(i16, 2);
impl_serialize_int!(u16, 2);
impl_serialize_int!(i32, 4);
impl_serialize_int!(u32, 4);
impl_serialize_int!(i64, 8);
impl_serialize_int!(u64, 8);
impl_serialize_int!(i128, 16);
impl_serialize_int!(u128, 16);
impl_serialize_int!(f32, 4);
impl_serialize_int!(f64, 8);

// ==================== usize ====================

/// `usize` 序列化为 8 字节 `u64` little-endian，确保跨平台兼容性。
///
/// `usize` is serialized as 8-byte `u64` little-endian for cross-platform compatibility.
impl AFastSerialize for usize {
    fn to_bytes(&self) -> Vec<u8> {
        (*self as u64).to_le_bytes().to_vec()
    }
}

impl AFastDeserialize for usize {
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let bytes = read_exact(data, 0, 8)?;
        let arr: [u8; 8] = bytes.try_into().unwrap();
        Ok((u64::from_le_bytes(arr) as usize, 8))
    }
}

// ==================== bool ====================

impl AFastSerialize for bool {
    /// 将布尔值序列化为 1 个字节：`true` 为 `0x01`，`false` 为 `0x00`。
    ///
    /// Serializes the boolean to 1 byte: `true` as `0x01`, `false` as `0x00`.
    fn to_bytes(&self) -> Vec<u8> {
        vec![if *self { 1 } else { 0 }]
    }
}

impl AFastDeserialize for bool {
    /// 从 1 个字节反序列化布尔值。仅接受 `0x00`（false）和 `0x01`（true）。
    ///
    /// Deserializes a boolean from 1 byte. Only accepts `0x00` (false) and `0x01` (true).
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let bytes = read_exact(data, 0, 1)?;
        match bytes[0] {
            0 => Ok((false, 1)),
            1 => Ok((true, 1)),
            v => Err(Error::deserialize(format!("Invalid bool value: {}", v))),
        }
    }
}

// ==================== 长度前缀辅助函数 / Length Prefix Helpers ====================

/// 将长度值写入缓冲区作为前缀字节。内部辅助函数。
///
/// Writes a length value into the buffer as prefix bytes. Internal helper.
///
/// 使用 [`LenInt`] 类型编码，little-endian 字节序。
///
/// Encoded using the [`LenInt`] type in little-endian byte order.
fn write_len(buf: &mut Vec<u8>, len: usize) {
    let v = len as LenInt;
    buf.extend(v.to_le_bytes());
}

/// 从字节切片中读取长度前缀。内部辅助函数。
///
/// Reads a length prefix from a byte slice. Internal helper.
///
/// # 返回值 / Returns
///
/// 返回 `(实际长度, 新偏移量)`，其中新偏移量 = 原偏移量 + `LEN_INT_SIZE`。
///
/// Returns `(actual_length, new_offset)` where new_offset = original_offset + `LEN_INT_SIZE`.
fn read_len(data: &[u8], offset: usize) -> Result<(usize, usize), Error> {
    let bytes = read_exact(data, offset, LEN_INT_SIZE)?;
    let arr: [u8; LEN_INT_SIZE] = bytes.try_into().unwrap();
    let len = LenInt::from_le_bytes(arr) as usize;
    Ok((len, offset + LEN_INT_SIZE))
}

// ==================== String ====================

impl AFastSerialize for String {
    /// 将字符串序列化为：`LenInt 长度前缀 + UTF-8 字节`。
    ///
    /// Serializes the string as: `LenInt length prefix + UTF-8 bytes`.
    ///
    /// 长度前缀表示 UTF-8 字节的长度（不是字符数）。
    ///
    /// The length prefix represents the number of UTF-8 bytes (not the character count).
    fn to_bytes(&self) -> Vec<u8> {
        let bytes = self.as_bytes();
        let mut result = Vec::with_capacity(LEN_INT_SIZE + bytes.len());
        write_len(&mut result, bytes.len());
        result.extend_from_slice(bytes);
        result
    }
}

impl AFastDeserialize for String {
    /// 从字节数据中反序列化字符串。
    ///
    /// Deserializes a string from byte data.
    ///
    /// 先读取 `LenInt` 长度前缀，再读取对应数量的 UTF-8 字节。
    ///
    /// First reads the `LenInt` length prefix, then reads the corresponding number
    /// of UTF-8 bytes.
    ///
    /// # 错误 / Errors
    ///
    /// 当字节数据不足或包含非法 UTF-8 序列时返回错误。
    ///
    /// Returns an error when there are insufficient bytes or the data contains
    /// invalid UTF-8 sequences.
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, offset) = read_len(data, 0)?;
        let bytes = read_exact(data, offset, len)?;
        let s = std::str::from_utf8(bytes)
            .map_err(|e| Error::deserialize(format!("Invalid UTF-8: {}", e)))?;
        Ok((s.to_owned(), offset + len))
    }
}

// ==================== Vec<T> ====================

impl<T: AFastSerialize> AFastSerialize for Vec<T> {
    /// 将向量序列化为：`LenInt 元素个数前缀 + 逐个元素的序列化数据`。
    ///
    /// Serializes the vector as: `LenInt element count prefix + serialized data for each element`.
    ///
    /// 每个元素调用其自身的 `to_bytes()` 方法，所有元素的序列化结果依次拼接。
    ///
    /// Each element's `to_bytes()` method is called, and all serialized results are
    /// concatenated sequentially.
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE + self.len());
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE + self.len());
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes_with(marker));
        }
        result
    }
}

impl<T: AFastDeserialize> AFastDeserialize for Vec<T> {
    /// 从字节数据中反序列化向量。
    ///
    /// Deserializes a vector from byte data.
    ///
    /// 先读取 `LenInt` 元素个数，再逐个反序列化元素。
    ///
    /// First reads the `LenInt` element count, then deserializes each element in order.
    ///
    /// # 错误 / Errors
    ///
    /// 当字节数据不足或任何元素反序列化失败时返回错误。
    ///
    /// Returns an error when there are insufficient bytes or any element fails to
    /// deserialize.
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes(&data[offset..])?;
            vec.push(item);
            offset += new_offset;
        }
        Ok((vec, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes_with(&data[offset..], marker)?;
            vec.push(item);
            offset += new_offset;
        }
        Ok((vec, offset))
    }
}

// ==================== Option<T> ====================

impl<T: AFastSerialize> AFastSerialize for Option<T> {
    /// 将 `Option<T>` 序列化。
    ///
    /// Serializes an `Option<T>`.
    ///
    /// - `None`：写入 `0x00`（1 字节）
    /// - `Some(value)`：写入 `0x01`（1 字节标记）+ 值的序列化数据
    ///
    /// - `None`: Writes `0x00` (1 byte)
    /// - `Some(value)`: Writes `0x01` (1 byte tag) + the serialized value data
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Some(val) => {
                let mut result = vec![1u8];
                result.extend(val.to_bytes());
                result
            }
            None => vec![0u8],
        }
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        match self {
            Some(val) => {
                let mut result = vec![1u8];
                result.extend(val.to_bytes_with(marker));
                result
            }
            None => vec![0u8],
        }
    }
}

impl<T: AFastDeserialize> AFastDeserialize for Option<T> {
    /// 从字节数据中反序列化 `Option<T>`。
    ///
    /// Deserializes an `Option<T>` from byte data.
    ///
    /// 先读取 1 字节标记：`0x00` 表示 `None`，`0x01` 表示 `Some` 并继续读取值。
    ///
    /// First reads a 1-byte tag: `0x00` means `None`, `0x01` means `Some` and
    /// continues to deserialize the value.
    ///
    /// # 错误 / Errors
    ///
    /// 当标记字节不是 `0x00` 或 `0x01` 时返回错误。
    ///
    /// Returns an error when the tag byte is neither `0x00` nor `0x01`.
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let bytes = read_exact(data, 0, 1)?;
        match bytes[0] {
            0 => Ok((None, 1)),
            1 => {
                let (val, new_offset) = T::from_bytes(&data[1..])?;
                Ok((Some(val), 1 + new_offset))
            }
            v => Err(Error::deserialize(format!("Invalid Option tag: {}", v))),
        }
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let bytes = read_exact(data, 0, 1)?;
        match bytes[0] {
            0 => Ok((None, 1)),
            1 => {
                let (val, new_offset) = T::from_bytes_with(&data[1..], marker)?;
                Ok((Some(val), 1 + new_offset))
            }
            v => Err(Error::deserialize(format!("Invalid Option tag: {}", v))),
        }
    }
}

// ==================== [T; N] 固定大小数组 / Fixed-size Arrays ====================

impl<T: AFastSerialize, const N: usize> AFastSerialize for [T; N] {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE + N);
        for item in self {
            result.extend(item.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE + N);
        for item in self {
            result.extend(item.to_bytes_with(marker));
        }
        result
    }
}

impl<T: AFastDeserialize + Default + Copy, const N: usize> AFastDeserialize for [T; N] {
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let mut arr = [T::default(); N];
        let mut offset = 0;
        for item in arr.iter_mut() {
            let (val, new_offset) = T::from_bytes(&data[offset..])?;
            *item = val;
            offset += new_offset;
        }
        Ok((arr, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let mut arr = [T::default(); N];
        let mut offset = 0;
        for item in arr.iter_mut() {
            let (val, new_offset) = T::from_bytes_with(&data[offset..], marker)?;
            *item = val;
            offset += new_offset;
        }
        Ok((arr, offset))
    }
}

// ==================== &str (仅序列化 / Serialize Only) ====================

impl AFastSerialize for &str {
    /// 将字符串切片序列化为：`LenInt 长度前缀 + UTF-8 字节`。
    ///
    /// Serializes the string slice as: `LenInt length prefix + UTF-8 bytes`.
    ///
    /// 与 `String` 的序列化格式完全一致，方便在不拥有所有权的情况下进行序列化。
    ///
    /// Identical to `String` serialization format, allowing serialization without
    /// taking ownership.
    fn to_bytes(&self) -> Vec<u8> {
        let bytes = self.as_bytes();
        let len = bytes.len() as LenInt;
        let mut result = len.to_le_bytes().to_vec();
        result.extend_from_slice(bytes);
        result
    }

    /// `&str` 是叶子类型，`to_bytes_with` 行为与 `to_bytes` 一致。
    ///
    /// `&str` is a leaf type; `to_bytes_with` behaves identically to `to_bytes`.
    fn to_bytes_with(&self, _marker: &str) -> Vec<u8> {
        self.to_bytes()
    }
}

// ==================== HashMap ====================

impl<K: AFastSerialize, V: AFastSerialize> AFastSerialize for std::collections::HashMap<K, V> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for (k, v) in self {
            result.extend(k.to_bytes());
            result.extend(v.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for (k, v) in self {
            result.extend(k.to_bytes_with(marker));
            result.extend(v.to_bytes_with(marker));
        }
        result
    }
}

impl<K: AFastDeserialize + Eq + std::hash::Hash, V: AFastDeserialize> AFastDeserialize
    for std::collections::HashMap<K, V>
{
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut map = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let (key, new_offset) = K::from_bytes(&data[offset..])?;
            offset += new_offset;
            let (val, new_offset) = V::from_bytes(&data[offset..])?;
            offset += new_offset;
            map.insert(key, val);
        }
        Ok((map, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut map = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let (key, new_offset) = K::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            let (val, new_offset) = V::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            map.insert(key, val);
        }
        Ok((map, offset))
    }
}

// ==================== HashSet ====================

impl<T: AFastSerialize> AFastSerialize for std::collections::HashSet<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes_with(marker));
        }
        result
    }
}

impl<T: AFastDeserialize + Eq + std::hash::Hash> AFastDeserialize for std::collections::HashSet<T> {
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut set = std::collections::HashSet::with_capacity(len);
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes(&data[offset..])?;
            offset += new_offset;
            set.insert(item);
        }
        Ok((set, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut set = std::collections::HashSet::with_capacity(len);
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            set.insert(item);
        }
        Ok((set, offset))
    }
}

// ==================== BTreeMap ====================

impl<K: AFastSerialize, V: AFastSerialize> AFastSerialize for std::collections::BTreeMap<K, V> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for (k, v) in self {
            result.extend(k.to_bytes());
            result.extend(v.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for (k, v) in self {
            result.extend(k.to_bytes_with(marker));
            result.extend(v.to_bytes_with(marker));
        }
        result
    }
}

impl<K: AFastDeserialize + Ord, V: AFastDeserialize> AFastDeserialize
    for std::collections::BTreeMap<K, V>
{
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut map = std::collections::BTreeMap::new();
        for _ in 0..len {
            let (key, new_offset) = K::from_bytes(&data[offset..])?;
            offset += new_offset;
            let (val, new_offset) = V::from_bytes(&data[offset..])?;
            offset += new_offset;
            map.insert(key, val);
        }
        Ok((map, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut map = std::collections::BTreeMap::new();
        for _ in 0..len {
            let (key, new_offset) = K::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            let (val, new_offset) = V::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            map.insert(key, val);
        }
        Ok((map, offset))
    }
}

// ==================== BTreeSet ====================

impl<T: AFastSerialize> AFastSerialize for std::collections::BTreeSet<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes());
        }
        result
    }

    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(LEN_INT_SIZE);
        write_len(&mut result, self.len());
        for item in self {
            result.extend(item.to_bytes_with(marker));
        }
        result
    }
}

impl<T: AFastDeserialize + Ord> AFastDeserialize for std::collections::BTreeSet<T> {
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut set = std::collections::BTreeSet::new();
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes(&data[offset..])?;
            offset += new_offset;
            set.insert(item);
        }
        Ok((set, offset))
    }

    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (len, mut offset) = read_len(data, 0)?;
        let mut set = std::collections::BTreeSet::new();
        for _ in 0..len {
            let (item, new_offset) = T::from_bytes_with(&data[offset..], marker)?;
            offset += new_offset;
            set.insert(item);
        }
        Ok((set, offset))
    }
}

// ==================== Tuple (元组) ====================

/// 为元组实现序列化/反序列化的宏。
///
/// Macro to implement serialization/deserialization for tuples.
macro_rules! impl_tuple {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        #[allow(non_snake_case)]
        impl<$first: AFastSerialize $(, $rest: AFastSerialize)*> AFastSerialize for ($first, $($rest,)*) {
            fn to_bytes(&self) -> Vec<u8> {
                let ($first, $($rest,)*) = self;
                let mut result = Vec::new();
                result.extend($first.to_bytes());
                $(result.extend($rest.to_bytes());)*
                result
            }
            fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
                let ($first, $($rest,)*) = self;
                let mut result = Vec::new();
                result.extend($first.to_bytes_with(marker));
                $(result.extend($rest.to_bytes_with(marker));)*
                result
            }
        }

        #[allow(non_snake_case)]
        impl<$first: AFastDeserialize $(, $rest: AFastDeserialize)*> AFastDeserialize for ($first, $($rest,)*) {
            fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
                let mut offset = 0;
                let ($first, new_offset) = <$first as AFastDeserialize>::from_bytes(&data[offset..])?;
                offset += new_offset;
                $(
                    let ($rest, new_offset) = <$rest as AFastDeserialize>::from_bytes(&data[offset..])?;
                    offset += new_offset;
                )*
                Ok((($first, $($rest,)*), offset))
            }
            fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
                let mut offset = 0;
                let ($first, new_offset) = <$first as AFastDeserialize>::from_bytes_with(&data[offset..], marker)?;
                offset += new_offset;
                $(
                    let ($rest, new_offset) = <$rest as AFastDeserialize>::from_bytes_with(&data[offset..], marker)?;
                    offset += new_offset;
                )*
                Ok((($first, $($rest,)*), offset))
            }
        }

        impl_tuple!($($rest),*);
    };
}

// ==================== Box<T> ====================

impl<T: AFastSerialize> AFastSerialize for Box<T> {
    fn to_bytes(&self) -> Vec<u8> {
        (**self).to_bytes()
    }
    fn to_bytes_with(&self, marker: &str) -> Vec<u8> {
        (**self).to_bytes_with(marker)
    }
}

impl<T: AFastDeserialize> AFastDeserialize for Box<T> {
    fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
        let (val, offset) = T::from_bytes(data)?;
        Ok((Box::new(val), offset))
    }
    fn from_bytes_with(data: &[u8], marker: &str) -> Result<(Self, usize), Error> {
        let (val, offset) = T::from_bytes_with(data, marker)?;
        Ok((Box::new(val), offset))
    }
}

// 通过 feature 控制支持的元组最大长度：
// tuple-8 → 最多 8 个元素
// tuple-16 → 最多 16 个元素（默认）
// tuple-32 → 最多 32 个元素
//
// Maximum tuple arity controlled by feature:
// tuple-8 → up to 8 elements
// tuple-16 → up to 16 elements (default)
// tuple-32 → up to 32 elements
#[cfg(all(
    feature = "tuple-8",
    not(any(feature = "tuple-16", feature = "tuple-32"))
))]
impl_tuple!(A, B, C, D, E, F, G, H);

#[cfg(all(feature = "tuple-16", not(feature = "tuple-32")))]
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(feature = "tuple-32")]
impl_tuple!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD,
    AE, AF
);
