//! # afastdata
//!
//! 高性能二进制序列化/反序列化库，通过 derive 宏为 Rust 类型自动生成序列化代码。
//!
//! A high-performance binary serialization/deserialization library that automatically
//! generates serialization code for Rust types via derive macros.
//!
//! ## 快速开始 / Quick Start
//!
//! ```rust
//! use afastdata::{AFastSerialize, AFastDeserialize};
//!
//! #[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
//! struct User {
//!     name: String,
//!     age: u32,
//!     email: Option<String>,
//! }
//!
//! let user = User {
//!     name: String::from("Alice"),
//!     age: 30,
//!     email: Some(String::from("alice@example.com")),
//! };
//!
//! // 序列化为字节数组 / Serialize to byte array
//! let bytes = user.to_bytes();
//!
//! // 从字节数组反序列化 / Deserialize from byte array
//! let (decoded, consumed) = User::from_bytes(&bytes).unwrap();
//! assert_eq!(user, decoded);
//! ```
//!
//! ## 功能特性 / Features
//!
//! - **零配置派生宏 / Zero-config derive macros**：`#[derive(AFastSerialize, AFastDeserialize)]`
//! - **支持的类型 / Supported types**：所有基本类型、`String`、`Vec<T>`、`Option<T>`、
//!   固定大小数组 `[T; N]`、嵌套结构体和枚举
//! - **泛型支持 / Generic support**：自动为泛型参数添加 trait 约束
//! - **可配置长度前缀 / Configurable length prefix**：默认 `u32`，可通过 `len-u64` feature
//!   切换为 `u64`
//!
//! ## 编码格式 / Encoding Format
//!
//! 所有数据使用 **小端序 (little-endian)** 编码。详细编码规则请参阅
//! [`afastdata_core`] 文档。
//!
//! All data uses **little-endian** encoding. For detailed encoding rules, see the
//! [`afastdata_core`] documentation.
//!
//! ## Feature Flags
//!
//! | Feature | 说明 / Description |
//! |---|---|
//! | `len-u64` | 将长度前缀从 `u32` 切换为 `u64` / Switch length prefix from `u32` to `u64` |
//!
//! ## Crate 结构 / Crate Structure
//!
//! - [`afastdata_core`]：核心 trait 定义和基本类型实现
//!   / Core trait definitions and primitive type implementations
//! - [`afastdata_macro`]：derive 宏实现
//!   / Derive macro implementations
//! - 本 crate 为统一入口，re-export 上述两个 crate 的所有公共项
//!   / This crate is the unified entry point, re-exporting all public items from the above

/// Re-export 核心 trait 和基本类型实现。
///
/// Re-exports core traits and primitive type implementations from [`afastdata_core`].
///
/// 包含以下主要项 / Includes the following main items:
/// - [`AFastSerialize`]：序列化 trait / Serialization trait
/// - [`AFastDeserialize`]：反序列化 trait / Deserialization trait
/// - [`LenInt`]：长度前缀类型别名 / Length prefix type alias
/// - [`LEN_INT_SIZE`]：长度前缀字节大小常量 / Length prefix byte size constant
pub use afastdata_core::*;

/// Re-export derive 宏。
///
/// Re-exports derive macros from [`afastdata_macro`].
///
/// 包含以下宏 / Includes the following macros:
/// - [`AFastSerialize`]：序列化 derive 宏 / Serialization derive macro
/// - [`AFastDeserialize`]：反序列化 derive 宏 / Deserialization derive macro
pub use afastdata_macro::*;
