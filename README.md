# afastdata

[![Crates.io](https://img.shields.io/crates/v/afastdata.svg)](https://crates.io/crates/afastdata)
[![docs.rs](https://docs.rs/afastdata/badge.svg)](https://docs.rs/afastdata)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

**[English](README_EN.md)** | 中文

高性能 Rust 二进制序列化/反序列化框架，通过 derive 宏为自定义类型自动生成序列化代码。

## 特性

- **零配置派生宏** — `#[derive(AFastSerialize, AFastDeserialize)]` 一行搞定
- **丰富的类型支持** — 基本类型、`String`、`Vec<T>`、`Option<T>`、`[T; N]`、嵌套结构体、枚举
- **泛型支持** — 自动为泛型参数添加 trait 约束
- **可配置长度前缀** — 默认 `u32`（最大 4GB），可通过 feature 切换为 `u64`
- **统一小端序** — 所有多字节数据使用 little-endian 编码
- **零外部依赖** — 运行时无第三方依赖

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
afastdata = "0.0.2"
```

如需 `u64` 长度前缀：

```toml
[dependencies]
afastdata = { version = "0.0.2", features = ["len-u64"] }
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

通过 `#[afast(...)]` 属性，为结构体字段添加校验规则。

### 示例

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

- `skip` or `skip(default)`：跳过此字段的序列化和反序列化，并使用默认值（调用传入的 default 函数或者给字段类型实现 Default trait）
- `gt(value, code, message)`：字段值必须大于 `value`，否则返回 `ValidateError`
- `gte(value, code, message)`：字段值必须大于等于 `value`，否则返回 `ValidateError`
- `lt(value, code, message)`：字段值必须小于 `value`，否则返回 `ValidateError`
- `lte(value, code, message)`：字段值必须小于等于 `value`，否则返回 `ValidateError`
- `len(min, max, code, message)`：字段长度必须在 `min` 和 `max` 之间（包含两端），字段类型 T 或者 Option<T> 中的 T 必须有 len():usize 方法，否则返回 `ValidateError`
- `of([v1, v2, ...], code, message)`：字段值必须在 `[v1, v2, ...]` 列表中，否则返回 `ValidateError`
- `func(name)`：调用外部函数 `name` 进行校验，函数签名为 `fn(value: &T, field: &str) -> Result<(), ValidateError>`，返回 `Ok(())` 则校验通过，否则返回 `ValidateError`

## 支持的类型

| 类型 | 序列化方式 | 字节数 |
|---|---|---|
| `i8`, `u8` | little-endian | 1 |
| `i16`, `u16` | little-endian | 2 |
| `i32`, `u32` | little-endian | 4 |
| `i64`, `u64` | little-endian | 8 |
| `i128`, `u128` | little-endian | 16 |
| `f32` | IEEE 754 little-endian | 4 |
| `f64` | IEEE 754 little-endian | 8 |
| `bool` | `0x00`=false, `0x01`=true | 1 |
| `String` | LenInt 长度前缀 + UTF-8 字节 | 变长 |
| `&str` | LenInt 长度前缀 + UTF-8 字节（仅序列化） | 变长 |
| `Vec<T>` | LenInt 元素个数 + 逐元素编码 | 变长 |
| `Option<T>` | 1 字节标记 + 数据（仅 Some） | 变长 |
| `[T; N]` | 逐元素编码，无长度前缀 | 固定 |
| 结构体 | 逐字段编码，无额外前缀 | 变长 |
| 枚举 | u32 变体索引 + 变体字段数据 | 变长 |

## 编码格式详解

### 结构体

所有字段按声明顺序依次序列化，无额外前缀：

```
[field1 bytes][field2 bytes][field3 bytes]...
```

### 枚举

先写入 `u32` 变体索引（从 0 开始，按声明顺序递增），再写入变体字段数据：

```
[u32 variant_index][field1 bytes][field2 bytes]...
```

Unit 变体只写入索引，无字段数据。

### 长度前缀

`String`、`Vec<T>` 等变长类型使用 `LenInt` 作为长度前缀：

- 默认：`u32` little-endian（4 字节，最大约 4GB）
- 启用 `len-u64` feature 后：`u64` little-endian（8 字节）

## Feature Flags

| Feature | 说明 |
|---|---|
| `len-u64` | 将长度前缀从 `u32` 切换为 `u64` |

## 项目结构

```
afastdata/
├── Cargo.toml              # Workspace 配置
├── README.md               # 中文文档（本文件）
├── README_EN.md            # English documentation
├── afastdata/              # 核心库 + 统一入口 crate
│   ├── Cargo.toml          # 含 `len-u64` feature
│   ├── src/lib.rs          # trait 定义 + 基本类型实现 + re-export derive 宏
│   └── examples/
│       └── basic.rs        # 基础使用示例
└── afastdata-macro/        # Proc-macro 库
    ├── Cargo.toml
    └── src/lib.rs          # AFastSerialize / AFastDeserialize derive 宏
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
