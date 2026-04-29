# afastdata

[![Crates.io](https://img.shields.io/crates/v/afastdata.svg)](https://crates.io/crates/afastdata)
[![docs.rs](https://docs.rs/afastdata/badge.svg)](https://docs.rs/afastdata)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

English | **[中文](README.md)**

A high-performance Rust binary serialization/deserialization framework that automatically generates serialization code for custom types via derive macros.

## Features

- **Zero-config derive macros** — `#[derive(AFastSerialize, AFastDeserialize)]` in one line
- **Rich type support** — Primitives, `String`, `Vec<T>`, `Option<T>`, `[T; N]`, `Box<T>`, tuples, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, nested structs, enums
- **Generic support** — Automatically adds trait bounds for generic parameters
- **Configurable length prefix** — Default `u32` (max 4GB), switchable to `u64` via feature flag
- **Configurable enum tags** — Default `u8`, switchable to `u16` or `u32` via feature flags
- **Configurable tuple support** — Default max 16 elements, switchable to 8 or 32 via feature flags
- **Uniform little-endian** — All multi-byte data uses little-endian encoding
- **Zero runtime dependencies** — No third-party dependencies at runtime

## Quick Start

### Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
afastdata = "0.0.4"
```

For `u64` length prefix or custom enum tag type:

```toml
[dependencies]
afastdata = { version = "0.0.4", features = ["len-u64", "tag-u16"] }
```

### Basic Usage

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

    // Serialize
    let bytes = user.to_bytes();

    // Deserialize
    let (decoded, consumed) = User::from_bytes(&bytes).unwrap();
    assert_eq!(user, decoded);
    println!("Consumed {} bytes", consumed);
}
```

### Enum Example

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

### Generic Struct

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

## Validation

You can add validation rules to struct fields and enum variant fields using the `#[afast(...)]` attribute.

### Struct Validation Example

```rust
use afastdata::{AFastDeserialize, AFastSerialize, ValidateError};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct A {
    #[afast(
        gt(10, 0, "${field} must be greater than 10"),
        lte(100, 0, "${field} must be less than or equal to 100")
    )]
    a: i64,

    #[afast(len(
        10,
        100,
        1,
        "${field} length must be between 10 and 100"
    ))]
    b: Option<String>,

    #[afast(func("v"))] // call v function to validate, parameter is `value` and `field name`
    c: i32,

    #[afast(skip("d"))] // call d function, #[afast(skip)] will call i64::default()
    e: i64,
}

fn v(value: &i32, field: &str) -> Result<(), ValidateError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidateError::new(
            2,
            format!("{} must be an even number, but got {}", field, value),
        ))
    }
}

fn d() -> i64 {
    123
}
```

### Enum Variant Validation Example

Validation rules also work on named-field and tuple enum variants:

```rust
use afastdata::{AFastDeserialize, AFastSerialize, ErrorKind};

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Command {
    // Named-field variant
    Login {
        #[afast(len(1, 32, 1001, "username ${field} length must be 1-32"))]
        username: String,
        #[afast(len(6, 128, 1002, "password ${field} length must be 6-128"))]
        password: String,
    },
    // Tuple variant
    Send(#[afast(gte(0, 2001, "value must be >= 0"))] i64),
}

fn main() {
    // Validation passes
    let cmd = Command::Login {
        username: String::from("alice"),
        password: String::from("secret123"),
    };
    let bytes = cmd.to_bytes();
    assert!(Command::from_bytes(&bytes).is_ok());

    // Validation fails: empty username
    let cmd = Command::Login {
        username: String::new(),
        password: String::from("secret123"),
    };
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1001, _)));
}
```

### Validate Attribute

Validation rules apply to both struct fields and enum variant fields (named and tuple variants):

- `skip` or `skip(default)`: skip this field's serialization and deserialization, and use default value (calling the default function or implementing Default trait for the field type)
- `gt(value, code, message)`: field value must be greater than `value`, otherwise return `ValidateError`
- `gte(value, code, message)`: field value must be greater than or equal to `value`, otherwise return `ValidateError`
- `lt(value, code, message)`: field value must be less than `value`, otherwise return `ValidateError`
- `lte(value, code, message)`: field value must be less than or equal to `value`, otherwise return `ValidateError`
- `len(min, max, code, message)`: field length must be between `min` and `max`, field type T or T in Option<T> must impl len():usize function, otherwise return `ValidateError`
- `of([v1, v2, ...], code, message)`: field value must be one of `[v1, v2, ...]`, otherwise return `ValidateError`
- `func(name)`: call external function `name` to perform validation, signature `fn(value: &T, field: &str) -> Result<(), ValidateError>`, return `Ok(())` if validation passes, otherwise return `ValidateError`

## Supported Types

| Type | Serialization | Size |
|---|---|---|
| `i8`, `u8` | little-endian | 1 byte |
| `i16`, `u16` | little-endian | 2 bytes |
| `i32`, `u32` | little-endian | 4 bytes |
| `i64`, `u64` | little-endian | 8 bytes |
| `i128`, `u128` | little-endian | 16 bytes |
| `f32` | IEEE 754 little-endian | 4 bytes |
| `f64` | IEEE 754 little-endian | 8 bytes |
| `bool` | `0x00`=false, `0x01`=true | 1 byte |
| `String` | LenInt prefix + UTF-8 bytes | Variable |
| `&str` | LenInt prefix + UTF-8 bytes (serialize only) | Variable |
| `Vec<T>` | LenInt element count + element-wise encoding | Variable |
| `Option<T>` | 1-byte tag + data (only when Some) | Variable |
| `[T; N]` | Element-wise encoding, no length prefix | Fixed |
| `(A, B, ...)` | Element-wise encoding, no length prefix | Fixed/Variable |
| `Box<T>` | Same as `T` | Same as `T` |
| `HashMap<K, V>` | LenInt entry count + key-value pair encoding | Variable |
| `HashSet<T>` | LenInt element count + element-wise encoding | Variable |
| `BTreeMap<K, V>` | LenInt entry count + key-value pair encoding | Variable |
| `BTreeSet<T>` | LenInt element count + element-wise encoding | Variable |
| Struct | Field-by-field encoding, no extra prefix | Variable |
| Enum | Tag(u8/u16/u32) variant index + variant field data | Variable |

## Encoding Format

### Struct

All fields are serialized in declaration order with no additional prefix:

```
[field1 bytes][field2 bytes][field3 bytes]...
```

### Enum

Writes a variant index (Tag type, default `u8`, switchable to `u16` or `u32` via feature flags, starting from 0, incrementing by declaration order), followed by the variant's field data:

```
[Tag variant_index][field1 bytes][field2 bytes]...
```

Unit variants only write the index, with no field data.

### Length Prefix

Variable-length types like `String` and `Vec<T>` use `LenInt` as the length prefix:

- Default: `u32` little-endian (4 bytes, max ~4GB)
- With `len-u64` feature: `u64` little-endian (8 bytes)

## Feature Flags

| Feature | Description | Default |
|---|---|---|
| `len-u64` | Switch the length prefix from `u32` to `u64` | No |
| `tag-u8` | Enum variant tag uses `u8` (1 byte, max 256 variants) | Yes |
| `tag-u16` | Enum variant tag uses `u16` (2 bytes, max 65536 variants) | No |
| `tag-u32` | Enum variant tag uses `u32` (4 bytes, max ~4.2 billion variants) | No |
| `tuple-8` | Tuple support up to 8 elements | No |
| `tuple-16` | Tuple support up to 16 elements | Yes |
| `tuple-32` | Tuple support up to 32 elements | No |

## Project Structure

```
afastdata/
├── Cargo.toml              # Workspace configuration
├── README.md               # Chinese documentation
├── README_EN.md            # English documentation (this file)
├── afastdata/              # Core library + unified entry crate
│   ├── Cargo.toml          # Contains `len-u64`, `tag-*`, `tuple-*` features
│   ├── src/lib.rs          # Trait definitions + primitive type implementations + re-exports derive macros
│   └── tests/
│       ├── derive_tests.rs     # Derive macro integration tests
│       └── primitive_tests.rs  # Primitive type serialization tests
└── afastdata-macro/        # Proc-macro library
    ├── Cargo.toml
    └── src/lib.rs          # AFastSerialize / AFastDeserialize derive macros
```

## Running the Example

```bash
cargo run --example basic -p afastdata
```

## Running Tests

```bash
cargo test --workspace
```

## License

MIT
