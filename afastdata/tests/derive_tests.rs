use afastdata::{AFastDeserialize, AFastSerialize};

// ==================== 校验测试 / Validation Tests ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct A {
    a: i32,
    #[afast(
        lte(10, 0, "b must be at least 10"),
    )]
    b: i64,
    c: String,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct B<T> {
    a: T,
    b: i32,
    c: String,
    d: Option<bool>,
    e: Vec<T>,
    f: C,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum C {
    Null,
    I32(i32),
    Struct(A),
    Tuple(i32, i64),
    Inner { a: i32, b: i64 },
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct D(i32, i64, String);

fn roundtrip<T: AFastSerialize + AFastDeserialize + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = val.to_bytes();
    let (decoded, offset) = T::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(*val, decoded);
}

#[test]
fn test_derive_struct_a() {
    let a = A {
        a: 42,
        b: -1000,
        c: String::from("hello"),
    };
    roundtrip(&a);
}

#[test]
fn test_derive_tuple_struct_d() {
    let d = D(1, 2, String::from("three"));
    roundtrip(&d);
}

#[test]
fn test_derive_enum_c() {
    roundtrip(&C::Null);
    roundtrip(&C::I32(42));
    roundtrip(&C::Tuple(1, 2));
    roundtrip(&C::Inner { a: 10, b: 20 });
    roundtrip(&C::Struct(A {
        a: 1,
        b: 2,
        c: String::from("nested"),
    }));
}

#[test]
fn test_derive_generic_struct_b() {
    let b = B {
        a: 99i32,
        b: 42,
        c: String::from("world"),
        d: Some(true),
        e: vec![1, 2, 3],
        f: C::I32(7),
    };
    roundtrip(&b);

    let b_none = B {
        a: 0i32,
        b: -1,
        c: String::from(""),
        d: None::<bool>,
        e: Vec::<i32>::new(),
        f: C::Null,
    };
    roundtrip(&b_none);
}

// ==================== 枚举变体校验测试 / Enum Variant Validation Tests ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Command {
    Unit,
    Login {
        #[afast(len(1, 32, 1001, "username ${field} length must be 1-32"))]
        username: String,
        #[afast(len(6, 128, 1002, "password ${field} length must be 6-128"))]
        password: String,
    },
    Transfer {
        #[afast(gte(1, 2001, "amount must be >= 1"))]
        amount: i64,
        #[afast(len(1, 64, 2002, "to ${field} length must be 1-64"))]
        to: String,
    },
    Config {
        #[afast(gte(1, 3001, "timeout must be >= 1"))]
        #[afast(lte(3600, 3002, "timeout must be <= 3600"))]
        timeout: i64,
    },
    // 元组变体校验 / Tuple variant validation
    Send(#[afast(gte(0, 4001, "value must be >= 0"))] i64),
    Echo(#[afast(len(1, 256, 4002, "msg ${field} length must be 1-256"))] String),
}

#[test]
fn test_enum_variant_valid_roundtrip() {
    // Login with valid data
    let login = Command::Login {
        username: String::from("alice"),
        password: String::from("secret123"),
    };
    roundtrip(&login);

    // Transfer with valid data
    let transfer = Command::Transfer {
        amount: 100,
        to: String::from("bob"),
    };
    roundtrip(&transfer);

    // Config with valid data
    let config = Command::Config { timeout: 30 };
    roundtrip(&config);

    // Unit variant
    roundtrip(&Command::Unit);

    // Tuple variant with valid data
    roundtrip(&Command::Send(42));
    roundtrip(&Command::Echo(String::from("hello")));
}

#[test]
fn test_enum_variant_validation_error() {
    use afastdata::ErrorKind;

    // Login with empty username (violates len(1, 32))
    let login = Command::Login {
        username: String::new(),
        password: String::from("secret123"),
    };
    let bytes = login.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1001, _)));

    // Login with short password (violates len(6, 128))
    let login = Command::Login {
        username: String::from("alice"),
        password: String::from("short"),
    };
    let bytes = login.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1002, _)));

    // Transfer with zero amount (violates gte(1))
    let transfer = Command::Transfer {
        amount: 0,
        to: String::from("bob"),
    };
    let bytes = transfer.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(2001, _)));

    // Transfer with empty to (violates len(1, 64))
    let transfer = Command::Transfer {
        amount: 100,
        to: String::new(),
    };
    let bytes = transfer.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(2002, _)));

    // Config with timeout 0 (violates gte(1))
    let config = Command::Config { timeout: 0 };
    let bytes = config.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3001, _)));

    // Config with timeout 7200 (violates lte(3600))
    let config = Command::Config { timeout: 7200 };
    let bytes = config.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3002, _)));

    // Send with negative value (violates gte(0))
    let send = Command::Send(-1);
    let bytes = send.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(4001, _)));

    // Echo with empty string (violates len(1, 256))
    let echo = Command::Echo(String::new());
    let bytes = echo.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(4002, _)));
}
