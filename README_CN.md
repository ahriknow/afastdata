# afastdata

[![CI](https://github.com/ahriknow/afastdata/actions/workflows/publish.yml/badge.svg)](https://github.com/ahriknow/afastdata/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/afastdata.svg)](https://crates.io/crates/afastdata)
[![docs.rs](https://docs.rs/afastdata/badge.svg)](https://docs.rs/afastdata)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

**[English](README.md)** | 中文

高性能 Rust 二进制序列化/反序列化框架，通过 derive 宏为自定义类型自动生成序列化代码。

## 特性

- **零配置派生宏** — `#[derive(AFastSerialize, AFastDeserialize)]`
- **丰富的类型支持** — 基本类型、`String`、`Vec<T>`、`Option<T>`、`[T; N]`、`Box<T>`、元组、`HashMap`、`HashSet`、`BTreeMap`、`BTreeSet`、嵌套结构体、枚举
- **泛型支持** — 自动为泛型参数添加 trait 约束
- **可配置长度前缀** — 默认 `u32`（最大 4GB），可通过 feature 切换为 `u64`
- **可配置枚举标签** — 默认 `u8`，可通过 feature 切换为 `u16` 或 `u32`
- **可配置元组支持** — 默认支持最多 16 个元素，可通过 feature 切换为 8 或 32
- **统一小端序** — 所有多字节数据使用 little-endian 编码
- **零外部依赖** — 运行时无第三方依赖

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
afastdata = "0.0.8"
```

如需 `u64` 长度前缀或自定义枚举标签类型：

```toml
[dependencies]
afastdata = { version = "0.0.8", features = ["len-u64", "tag-u16"] }
```

### 基本用法

```rust
use afastdata::{AFastSerialize, AFastDeserialize};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    let user = User {
        name: String::from("Alice"),
        age: 30,
        email: Some(String::from("alice@example.com")),
    };

    // 序列化
    let bytes = user.to_bytes();

    // 反序列化
    let (decoded, consumed) = User::from_bytes(&bytes).unwrap();
    assert_eq!(user, decoded);
    println!("消耗 {} 字节", consumed);
}
```

### 枚举示例

```rust
use afastdata::{AFastSerialize, AFastDeserialize};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Command {
    Ping,
    Send { to: String, message: String },
    Broadcast(Vec<String>),
}

fn main() {
    let cmd = Command::Send {
        to: String::from("Bob"),
        message: String::from("Hello!"),
    };

    let bytes = cmd.to_bytes();
    let (decoded, _) = Command::from_bytes(&bytes).unwrap();
    assert_eq!(cmd, decoded);
}
```

### 泛型结构体

```rust
use afastdata::{AFastSerialize, AFastDeserialize};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Response<T> {
    code: u32,
    data: Option<T>,
    message: String,
}

fn main() {
    let resp = Response {
        code: 200,
        data: Some(vec![1i32, 2, 3]),
        message: String::from("ok"),
    };

    let bytes = resp.to_bytes();
    let (decoded, _) = Response::<Vec<i32>>::from_bytes(&bytes).unwrap();
    assert_eq!(resp, decoded);
}
```

## 数据校验

通过 `#[afast(...)]` 属性，为结构体字段和枚举变体字段添加校验规则。

### 结构体校验示例

```rust
use afastdata::{AFastDeserialize, AFastSerialize, ValidateError};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct A {
    #[afast(
        gt(10, 0, "${field} 必须大于 10"),
        lte(100, 0, "${field} 必须小于等于 100")
    )]
    a: i64,

    #[afast(len(
        10,
        100,
        1,
        "${field} 长度必须大于等于 10 且小于等于 100"
    ))]
    b: Option<String>,

    #[afast(func("v"))] // 调用 v 函数进行校验, 参数为 `值` 和 `字段名`
    c: i32,

    #[afast(skip("d"))] // 调用 d 函数, #[afast(skip)] 将调用 i64::default()
    e: i64,
}

fn v(value: &i32, field: &str) -> Result<(), ValidateError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidateError::new(
            2,
            format!("{} 必须是偶数，但实际为 {}", field, value),
        ))
    }
}

fn d() -> i64 {
    123
}
```

### 校验规则

校验规则适用于结构体字段和枚举变体字段（命名字段和元组变体）：

- `skip` 或 `skip("default_fn")`：跳过此字段的序列化和反序列化。反序列化时使用默认值（调用传入的函数或字段类型的 `Default::default()`）
- `skip_with("marker")` 或 `skip_with("marker", "default_fn")`：条件跳过。当 `to_bytes_with` / `from_bytes_with` 传入的 marker 与字段上的 marker 匹配时，该字段被跳过。不匹配时正常序列化/反序列化。marker 会自动传播到嵌套类型——如果某个结构体字段包含另一个带有 `skip_with` 字段的结构体，内部字段也会响应同一个 marker
- `gt(value, code, message)`：字段值必须大于 `value`（支持整数和浮点数），否则返回 `ValidateError`
- `gte(value, code, message)`：字段值必须大于等于 `value`，否则返回 `ValidateError`
- `lt(value, code, message)`：字段值必须小于 `value`，否则返回 `ValidateError`
- `lte(value, code, message)`：字段值必须小于等于 `value`，否则返回 `ValidateError`
- `len(min, max, code, message)`：字段长度必须在 `min` 和 `max` 之间。`min` 或 `max` 传 `-1` 表示不限制该边界
- `of([v1, v2, ...], code, message)`：字段值必须在列表中
- `func(name)`：调用外部函数进行校验，签名 `fn(value: &T, field: &str) -> Result<(), ValidateError>`

> **类型检查**：校验规则在编译期检查字段类型兼容性。

## 条件序列化（`skip_with`）

`skip_with` 允许根据 marker 条件跳过字段，适用于同一类型在不同场景下的序列化策略：

```rust
use afastdata::{AFastSerialize, AFastDeserialize};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Packet {
    header: u32,
    #[afast(skip_with("cache"))]
    payload: Vec<u8>,
    #[afast(skip_with("cache", "default_checksum"))]
    checksum: u64,
}

fn default_checksum() -> u64 { 0 }

fn main() {
    let pkt = Packet {
        header: 1,
        payload: vec![1, 2, 3],
        checksum: 0xABCD,
    };

    // 正常序列化（包含所有字段）
    let full = pkt.to_bytes();

    // 条件序列化（跳过 marker="cache" 的字段）
    let cached = pkt.to_bytes_with("cache");
    // cached 只包含 header，比 full 短

    // 条件反序列化
    let (decoded, _) = Packet::from_bytes_with(&cached, "cache").unwrap();
    assert_eq!(decoded.header, 1);
    assert_eq!(decoded.payload, Vec::new());     // Default::default()
    assert_eq!(decoded.checksum, 0);             // default_checksum()
}
```

`to_bytes_with(marker)` 和 `from_bytes_with(data, marker)` 接受一个 marker 字符串：
- marker 与字段 `skip_with` 的标记匹配时：序列化跳过该字段，反序列化用默认值/自定义函数填充
- marker 不匹配时：行为与 `to_bytes()` / `from_bytes()` 完全一致
- **嵌套传播** — 使用 `_with` 方法序列化/反序列化时，marker 会自动传播到所有嵌套类型。如果内部结构体也有 `skip_with` 字段，它们会响应同一个 marker
- 对基本类型（`i32`、`String`、`Vec<T>` 等），`_with` 方法默认实现直接调用普通方法

### 嵌套类型示例

`skip_with` 在嵌套结构体之间自动生效，无需特殊配置：

```rust
use afastdata::{AFastSerialize, AFastDeserialize};

#[derive(AFastSerialize, AFastDeserialize, Debug, Default, PartialEq)]
struct Inner {
    id: u32,
    #[afast(skip_with("cache"))]
    data: Vec<u8>,
    name: String,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Outer {
    header: u32,
    inner: Inner,
    footer: u32,
}

fn main() {
    let outer = Outer {
        header: 1,
        inner: Inner {
            id: 42,
            data: vec![1, 2, 3, 4, 5],
            name: String::from("test"),
        },
        footer: 99,
    };

    // 完整序列化——包含 inner.data
    let full = outer.to_bytes();

    // 条件序列化——inner.data 通过 marker 传播被跳过
    let cached = outer.to_bytes_with("cache");
    assert!(cached.len() < full.len());

    // 条件反序列化——inner.data 用 Default::default() 填充
    let (decoded, _) = Outer::from_bytes_with(&cached, "cache").unwrap();
    assert_eq!(decoded.inner.id, 42);
    assert_eq!(decoded.inner.data, Vec::new());
    assert_eq!(decoded.inner.name, String::from("test"));
}
```

## 字段名安全性

生成代码中的所有内部变量均以 `__afast_` 为前缀，避免与用户字段名冲突。可以安全使用 `data`、`offset`、`bytes` 等名称作为字段名：

```rust
#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Record {
    data: Vec<u8>,    // 不会与内部 data 参数冲突
    offset: u64,      // 不会与内部 offset 变量冲突
    bytes: String,    // 不会与内部 bytes 变量冲突
}
```

## 支持的类型

| 类型 | 序列化方式 | 字节数 |
|---|---|---|
| `i8`, `u8` | little-endian | 1 |
| `i16`, `u16` | little-endian | 2 |
| `i32`, `u32` | little-endian | 4 |
| `i64`, `u64` | little-endian | 8 |
| `usize` | u64 little-endian（跨平台兼容） | 8 |
| `i128`, `u128` | little-endian | 16 |
| `f32` | IEEE 754 little-endian | 4 |
| `f64` | IEEE 754 little-endian | 8 |
| `bool` | `0x00`=false, `0x01`=true | 1 |
| `String` | LenInt 长度前缀 + UTF-8 字节 | 变长 |
| `&str` | LenInt 长度前缀 + UTF-8 字节（仅序列化） | 变长 |
| `Vec<T>` | LenInt 元素个数 + 逐元素编码 | 变长 |
| `Option<T>` | 1 字节标记 + 数据（仅 Some） | 变长 |
| `[T; N]` | 逐元素编码，无长度前缀 | 固定 |
| `(A, B, ...)` | 逐元素编码，无长度前缀 | 固定/变长 |
| `Box<T>` | 与 `T` 相同 | 与 `T` 相同 |
| `HashMap<K, V>` | LenInt 键值对数 + 逐对编码 | 变长 |
| `HashSet<T>` | LenInt 元素个数 + 逐元素编码 | 变长 |
| `BTreeMap<K, V>` | LenInt 键值对数 + 逐对编码 | 变长 |
| `BTreeSet<T>` | LenInt 元素个数 + 逐元素编码 | 变长 |
| 结构体 | 逐字段编码，无额外前缀 | 变长 |
| 枚举 | Tag(u8/u16/u32) 变体索引 + 变体字段数据 | 变长 |

## 编码格式详解

### 结构体

所有字段按声明顺序依次序列化，无额外前缀：

```
[field1 bytes][field2 bytes][field3 bytes]...
```

### 枚举

先写入变体索引（Tag 类型，默认 `u8`，可通过 feature 切换为 `u16` 或 `u32`，从 0 开始按声明顺序递增），再写入变体字段数据：

```
[Tag variant_index][field1 bytes][field2 bytes]...
```

Unit 变体只写入索引，无字段数据。

### 长度前缀

`String`、`Vec<T>` 等变长类型使用 `LenInt` 作为长度前缀：

- 默认：`u32` little-endian（4 字节，最大约 4GB）
- 启用 `len-u64` feature 后：`u64` little-endian（8 字节）

## Feature Flags

| Feature | 说明 | 默认 |
|---|---|---|
| `len-u64` | 将长度前缀从 `u32` 切换为 `u64` | 否 |
| `tag-u8` | 枚举变体标签使用 `u8`（1 字节，最多 256 个变体） | 是 |
| `tag-u16` | 枚举变体标签使用 `u16`（2 字节，最多 65536 个变体） | 否 |
| `tag-u32` | 枚举变体标签使用 `u32`（4 字节，最多约 42 亿个变体） | 否 |
| `tuple-8` | 元组支持最多 8 个元素 | 否 |
| `tuple-16` | 元组支持最多 16 个元素 | 是 |
| `tuple-32` | 元组支持最多 32 个元素 | 否 |

## 项目结构

```
afastdata/
├── Cargo.toml                  # Workspace 配置
├── README.md                   # English documentation (default)
├── README_CN.md                # 中文文档（本文件）
├── afastdata/                  # 核心库 + 统一入口 crate
│   ├── Cargo.toml              # 含 `len-u64`、`tag-*`、`tuple-*` features
│   ├── src/lib.rs              # trait 定义 + 基本类型实现 + re-export derive 宏
│   └── tests/
│       ├── derive_tests.rs     # 派生宏集成测试
│       └── primitive_tests.rs  # 基本类型序列化测试
└── afastdata-macro/            # Proc-macro 库
    ├── Cargo.toml
    └── src/lib.rs              # AFastSerialize / AFastDeserialize derive 宏
```

## 运行示例

```bash
cargo run --example basic -p afastdata
```

## 运行测试

```bash
cargo test --workspace
```

## License

MIT
