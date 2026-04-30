use afastdata::{AFastDeserialize, AFastSerialize, ErrorKind, ValidateError};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn roundtrip<T: AFastSerialize + AFastDeserialize + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = val.to_bytes();
    let (decoded, offset) = T::from_bytes(&bytes).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(*val, decoded);
}

// ==================== 基础结构体 / Basic Structs ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct UnitStruct;

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct TupleStruct(i32, String, bool);

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct NamedStruct {
    id: u64,
    name: String,
    active: bool,
}

#[test]
fn test_unit_struct() {
    roundtrip(&UnitStruct);
}

#[test]
fn test_tuple_struct() {
    roundtrip(&TupleStruct(42, String::from("hello"), true));
    roundtrip(&TupleStruct(-1, String::new(), false));
}

#[test]
fn test_named_struct() {
    let s = NamedStruct {
        id: 12345,
        name: String::from("test"),
        active: true,
    };
    roundtrip(&s);
}

// ==================== 枚举 / Enums ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum SimpleEnum {
    A,
    B,
    C,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum TupleEnum {
    Null,
    Int(i32),
    Pair(i64, String),
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum NamedEnum {
    Unit,
    Struct { x: f64, y: f64 },
    WithString { msg: String },
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum MixedEnum {
    Unit,
    Tuple(i32, String),
    Named { a: u8, b: Vec<i32> },
}

#[test]
fn test_unit_enum() {
    roundtrip(&SimpleEnum::A);
    roundtrip(&SimpleEnum::B);
    roundtrip(&SimpleEnum::C);
}

#[test]
fn test_tuple_enum() {
    roundtrip(&TupleEnum::Null);
    roundtrip(&TupleEnum::Int(99));
    roundtrip(&TupleEnum::Pair(-1, String::from("data")));
}

#[test]
fn test_named_enum() {
    roundtrip(&NamedEnum::Unit);
    roundtrip(&NamedEnum::Struct { x: 1.5, y: 2.5 });
    roundtrip(&NamedEnum::WithString {
        msg: String::from("hello"),
    });
}

#[test]
fn test_mixed_enum() {
    roundtrip(&MixedEnum::Unit);
    roundtrip(&MixedEnum::Tuple(10, String::from("t")));
    roundtrip(&MixedEnum::Named {
        a: 255,
        b: vec![1, 2, 3],
    });
}

// ==================== 泛型 / Generics ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct SingleGeneric<T> {
    value: T,
    label: String,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct MultiGeneric<T, U> {
    first: T,
    second: U,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct NestedGeneric<T> {
    inner: SingleGeneric<T>,
    count: u32,
}

#[test]
fn test_single_generic() {
    roundtrip(&SingleGeneric {
        value: 42i32,
        label: String::from("num"),
    });
    roundtrip(&SingleGeneric {
        value: String::from("val"),
        label: String::from("str"),
    });
}

#[test]
fn test_multi_generic() {
    roundtrip(&MultiGeneric {
        first: 1i32,
        second: String::from("two"),
    });
    roundtrip(&MultiGeneric {
        first: true,
        second: 3.14f64,
    });
}

#[test]
fn test_nested_generic() {
    roundtrip(&NestedGeneric {
        inner: SingleGeneric {
            value: 100u64,
            label: String::from("inner"),
        },
        count: 5,
    });
}

// ==================== 嵌套类型 / Nested Types ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Address {
    city: String,
    zip: u32,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u8,
    address: Address,
    tags: Vec<String>,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Company {
    name: String,
    employees: Vec<Person>,
    metadata: HashMap<String, String>,
}

#[test]
fn test_nested_struct() {
    let person = Person {
        name: String::from("Alice"),
        age: 30,
        address: Address {
            city: String::from("Beijing"),
            zip: 100000,
        },
        tags: vec![String::from("admin"), String::from("dev")],
    };
    roundtrip(&person);
}

#[test]
fn test_deeply_nested() {
    let company = Company {
        name: String::from("Acme"),
        employees: vec![
            Person {
                name: String::from("Bob"),
                age: 25,
                address: Address {
                    city: String::from("Shanghai"),
                    zip: 200000,
                },
                tags: vec![],
            },
            Person {
                name: String::from("Carol"),
                age: 28,
                address: Address {
                    city: String::from("Shenzhen"),
                    zip: 518000,
                },
                tags: vec![String::from("lead")],
            },
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert(String::from("founded"), String::from("2020"));
            m
        },
    };
    roundtrip(&company);
}

// ==================== Option / Vec / Array / Tuple 字段 ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Collections {
    optional: Option<i32>,
    optional_string: Option<String>,
    list: Vec<u8>,
    matrix: [[i32; 3]; 2],
    pair: (i32, String),
    triple: (u8, u16, u32),
}

#[test]
fn test_collections_in_struct() {
    let c = Collections {
        optional: Some(42),
        optional_string: None,
        list: vec![1, 2, 3, 4, 5],
        matrix: [[1, 2, 3], [4, 5, 6]],
        pair: (10, String::from("pair")),
        triple: (1, 2, 3),
    };
    roundtrip(&c);

    let c2 = Collections {
        optional: None,
        optional_string: Some(String::from("yes")),
        list: vec![],
        matrix: [[0; 3]; 2],
        pair: (0, String::new()),
        triple: (0, 0, 0),
    };
    roundtrip(&c2);
}

// ==================== HashMap / BTreeMap / HashSet / BTreeSet 字段 ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct MapsAndSets {
    hash_map: HashMap<String, i32>,
    btree_map: BTreeMap<u32, String>,
    hash_set: HashSet<u64>,
    btree_set: BTreeSet<String>,
}

#[test]
fn test_maps_and_sets_in_struct() {
    let mut hm = HashMap::new();
    hm.insert(String::from("a"), 1);
    hm.insert(String::from("b"), 2);

    let mut bm = BTreeMap::new();
    bm.insert(10u32, String::from("ten"));
    bm.insert(20, String::from("twenty"));

    let mut hs = HashSet::new();
    hs.insert(100u64);
    hs.insert(200);

    let mut bs = BTreeSet::new();
    bs.insert(String::from("x"));
    bs.insert(String::from("y"));

    let m = MapsAndSets {
        hash_map: hm,
        btree_map: bm,
        hash_set: hs,
        btree_set: bs,
    };
    roundtrip(&m);
}

// ==================== 枚举嵌套 / Enum Nesting ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle(f64, f64, f64),
    Empty,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Canvas {
    shapes: Vec<Shape>,
    active: Option<Shape>,
    background: String,
}

#[test]
fn test_enum_in_struct() {
    let canvas = Canvas {
        shapes: vec![
            Shape::Circle { radius: 5.0 },
            Shape::Rectangle {
                width: 10.0,
                height: 20.0,
            },
            Shape::Triangle(3.0, 4.0, 5.0),
            Shape::Empty,
        ],
        active: Some(Shape::Circle { radius: 1.0 }),
        background: String::from("#fff"),
    };
    roundtrip(&canvas);
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Container {
    Int(i32),
    List(Vec<String>),
    Map(HashMap<String, i32>),
    Nested { inner: Box<Container> },
}

#[test]
fn test_recursive_enum() {
    roundtrip(&Container::Int(42));
    roundtrip(&Container::List(vec![
        String::from("a"),
        String::from("b"),
    ]));
    roundtrip(&Container::Map({
        let mut m = HashMap::new();
        m.insert(String::from("key"), 100);
        m
    }));
    roundtrip(&Container::Nested {
        inner: Box::new(Container::Int(99)),
    });
}

// ==================== Skip 属性 / Skip Attribute ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct WithSkip {
    visible: i32,
    #[afast(skip)]
    _hidden: String,
    also_visible: bool,
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct WithSkipFn {
    x: i32,
    #[afast(skip("default_name"))]
    _computed: String,
}

fn default_name() -> String {
    String::from("default")
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct TupleWithSkip(
    i32,
    #[afast(skip)] String,
    bool,
);

#[test]
fn test_skip_named_field() {
    let s = WithSkip {
        visible: 1,
        _hidden: String::from("secret"),
        also_visible: true,
    };
    let bytes = s.to_bytes();
    let (decoded, _) = WithSkip::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.visible, 1);
    assert_eq!(decoded._hidden, String::new());
    assert_eq!(decoded.also_visible, true);
}

#[test]
fn test_skip_with_fn() {
    let s = WithSkipFn {
        x: 42,
        _computed: String::from("ignored"),
    };
    let bytes = s.to_bytes();
    let (decoded, _) = WithSkipFn::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.x, 42);
    assert_eq!(decoded._computed, String::from("default"));
}

#[test]
fn test_skip_tuple_field() {
    let s = TupleWithSkip(1, String::from("hidden"), true);
    let bytes = s.to_bytes();
    let (decoded, _) = TupleWithSkip::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.0, 1);
    assert_eq!(decoded.1, String::new());
    assert_eq!(decoded.2, true);
}

// ==================== 枚举变体 Skip / Enum Variant Skip ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum EnumWithSkip {
    Unit,
    Named {
        id: u32,
        #[afast(skip)]
        _internal: String,
    },
}

#[test]
fn test_enum_variant_skip() {
    let e = EnumWithSkip::Named {
        id: 10,
        _internal: String::from("secret"),
    };
    let bytes = e.to_bytes();
    assert_eq!(bytes, vec![1, 10, 0, 0, 0]);
    let (decoded, _) = <EnumWithSkip as AFastDeserialize>::from_bytes(&bytes).unwrap();
    match decoded {
        EnumWithSkip::Named { id, _internal } => {
            assert_eq!(id, 10);
            assert_eq!(_internal, String::new());
        }
        _ => panic!("expected Named variant"),
    }
}

// ==================== 校验属性 / Validation ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Validated {
    #[afast(gte(0, 1001, "age must be >= 0"))]
    #[afast(lte(150, 1002, "age must be <= 150"))]
    age: i64,
    #[afast(len(1, 50, 1003, "name ${field} must be 1-50 chars"))]
    name: String,
    #[afast(of([1, 2, 3, 5, 8], 1006, "priority must be 1,2,3,5,8"))]
    priority: i64,
}

fn validate_positive(val: &i32, field: &str) -> Result<(), ValidateError> {
    if *val < 0 {
        Err(ValidateError::new(
            1007,
            format!("{} must be positive", field),
        ))
    } else {
        Ok(())
    }
}

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct FuncValidated {
    #[afast(func("validate_positive"))]
    value: i32,
}

#[test]
fn test_validation_pass() {
    let v = Validated {
        age: 25,
        name: String::from("Alice"),

        priority: 5,
    };
    roundtrip(&v);
}

#[test]
fn test_validation_fail_gte() {
    let v = Validated {
        age: -1,
        name: String::from("Alice"),

        priority: 5,
    };
    let bytes = v.to_bytes();
    let err = Validated::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1001, _)));
}

#[test]
fn test_validation_fail_lte() {
    let v = Validated {
        age: 200,
        name: String::from("Alice"),

        priority: 5,
    };
    let bytes = v.to_bytes();
    let err = Validated::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1002, _)));
}

#[test]
fn test_validation_fail_len() {
    let v = Validated {
        age: 25,
        name: String::new(),

        priority: 5,
    };
    let bytes = v.to_bytes();
    let err = Validated::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1003, _)));
}

#[test]
fn test_validation_fail_of() {
    let v = Validated {
        age: 25,
        name: String::from("Alice"),

        priority: 4,
    };
    let bytes = v.to_bytes();
    let err = Validated::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1006, _)));
}

#[test]
fn test_validation_func_pass() {
    let v = FuncValidated { value: 10 };
    roundtrip(&v);
}

#[test]
fn test_validation_func_fail() {
    let v = FuncValidated { value: -1 };
    let bytes = v.to_bytes();
    let err = FuncValidated::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(1007, _)));
}

// ==================== Option + Len 校验 / Option Len Validation ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct OptionalLen {
    #[afast(len(0, 10, 2001, "tag ${field} must be 0-10 chars"))]
    tag: Option<String>,
}

#[test]
fn test_option_len_validation_pass() {
    roundtrip(&OptionalLen {
        tag: Some(String::from("hello")),
    });
    roundtrip(&OptionalLen { tag: None });
}

#[test]
fn test_option_len_validation_fail() {
    let v = OptionalLen {
        tag: Some(String::from("this is way too long")),
    };
    let bytes = v.to_bytes();
    let err = OptionalLen::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(2001, _)));
}

// ==================== 枚举变体校验 / Enum Variant Validation ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
enum Command {
    Ping,
    Login {
        #[afast(len(1, 32, 3001, "username must be 1-32"))]
        username: String,
        #[afast(len(6, 128, 3002, "password must be 6-128"))]
        password: String,
    },
    Transfer {
        #[afast(gte(1, 3003, "amount must be >= 1"))]
        amount: i64,
    },
    Send(#[afast(gte(0, 3004, "value must be >= 0"))] i64),
    Echo(#[afast(len(1, 256, 3005, "msg must be 1-256"))] String),
}

#[test]
fn test_enum_variant_validation_pass() {
    roundtrip(&Command::Ping);
    roundtrip(&Command::Login {
        username: String::from("alice"),
        password: String::from("secret123"),
    });
    roundtrip(&Command::Transfer { amount: 100 });
    roundtrip(&Command::Send(42));
    roundtrip(&Command::Echo(String::from("hello")));
}

#[test]
fn test_enum_variant_validation_fail() {
    // Empty username
    let cmd = Command::Login {
        username: String::new(),
        password: String::from("secret123"),
    };
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3001, _)));

    // Short password
    let cmd = Command::Login {
        username: String::from("alice"),
        password: String::from("short"),
    };
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3002, _)));

    // Zero amount
    let cmd = Command::Transfer { amount: 0 };
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3003, _)));

    // Negative send
    let cmd = Command::Send(-1);
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3004, _)));

    // Empty echo
    let cmd = Command::Echo(String::new());
    let bytes = cmd.to_bytes();
    let err = Command::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(3005, _)));
}

// ==================== 复杂组合 / Complex Combinations ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct Complex {
    id: u64,
    tags: Vec<String>,
    score: Option<f64>,
    metadata: HashMap<String, Vec<i32>>,
    shapes: Vec<Shape>,
    position: (f32, f32, f32),
    matrix: [[u8; 4]; 4],
    active: bool,
}

#[test]
fn test_complex_combination() {
    let mut meta = HashMap::new();
    meta.insert(String::from("scores"), vec![10, 20, 30]);
    meta.insert(String::from("levels"), vec![1, 2]);

    let c = Complex {
        id: 999,
        tags: vec![String::from("rust"), String::from("fast")],
        score: Some(98.5),
        metadata: meta,
        shapes: vec![
            Shape::Circle { radius: 3.0 },
            Shape::Empty,
        ],
        position: (1.0, 2.0, 3.0),
        matrix: [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1],
        ],
        active: true,
    };
    roundtrip(&c);
}

// ==================== 未知枚举变体 / Unknown Variant ====================

#[test]
fn test_unknown_variant_tag() {
    let data = 99u32.to_le_bytes();
    let err = SimpleEnum::from_bytes(&data).unwrap_err();
    assert!(format!("{}", err).contains("Unknown variant tag"));
}

// ==================== f64 校验测试 / f64 Validation Tests ====================

#[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
struct FloatStruct {
    #[afast(gte(0.0, 5001, "value must be >= 0.0"))]
    #[afast(lte(100.0, 5002, "value must be <= 100.0"))]
    score: f64,
}

#[test]
fn test_f64_validation_valid() {
    let s = FloatStruct { score: 50.5 };
    roundtrip(&s);
}

#[test]
fn test_f64_validation_error() {
    use afastdata::ErrorKind;

    // below minimum
    let s = FloatStruct { score: -1.0 };
    let bytes = s.to_bytes();
    let err = FloatStruct::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(5001, _)));

    // above maximum
    let s = FloatStruct { score: 101.0 };
    let bytes = s.to_bytes();
    let err = FloatStruct::from_bytes(&bytes).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ValidateError(5002, _)));
}
