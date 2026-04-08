# afastdata

[![Crates.io](https://img.shields.io/crates/v/afastdata.svg)](https://crates.io/crates/afastdata)
[![docs.rs](https://docs.rs/afastdata/badge.svg)](https://docs.rs/afastdata)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

English | **[中文](README.md)**

A high-performance Rust binary serialization/deserialization framework that automatically generates serialization code for custom types via derive macros.

## Features

- **Zero-config derive macros** — `#[derive(AFastSerialize, AFastDeserialize)]` in one line
- **Rich type support** — Primitives, `String`, `Vec<T>`, `Option<T>`, `[T; N]`, nested structs, enums
- **Generic support** — Automatically adds trait bounds for generic parameters
- **Configurable length prefix** — Default `u32` (max 4GB), switchable to `u64` via feature flag
- **Uniform little-endian** — All multi-byte data uses little-endian encoding
- **Zero runtime dependencies** — No third-party dependencies at runtime

## Quick Start

### Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
afastdata = "0.0.1"
```

For `u64` length prefix support:

```toml
[dependencies]
afastdata = { version = "0.0.1", features = ["len-u64"] }
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
| Struct | Field-by-field encoding, no extra prefix | Variable |
| Enum | `u32` variant index + variant field data | Variable |

## Encoding Format

### Struct

All fields are serialized in declaration order with no additional prefix:

```
[field1 bytes][field2 bytes][field3 bytes]...
```

### Enum

Writes a `u32` variant index (starting from 0, incrementing by declaration order), followed by the variant's field data:

```
[u32 variant_index][field1 bytes][field2 bytes]...
```

Unit variants only write the index, with no field data.

### Length Prefix

Variable-length types like `String` and `Vec<T>` use `LenInt` as the length prefix:

- Default: `u32` little-endian (4 bytes, max ~4GB)
- With `len-u64` feature: `u64` little-endian (8 bytes)

## Feature Flags

| Feature | Description |
|---|---|
| `len-u64` | Switch the length prefix from `u32` to `u64` |

## Project Structure

```
afastdata/
├── Cargo.toml              # Workspace configuration
├── README.md               # Chinese documentation
├── README_EN.md            # English documentation (this file)
├── afastdata/              # Unified entry crate
│   ├── Cargo.toml
│   ├── src/lib.rs          # Re-exports core traits and derive macros
│   └── examples/
│       └── basic.rs        # Basic usage example
├── afastdata-core/         # Core library
│   ├── Cargo.toml          # Contains `len-u64` feature
│   └── src/lib.rs          # Trait definitions + primitive type implementations
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
