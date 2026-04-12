use afastdata_core::{AFastDeserialize, AFastSerialize};
use afastdata_macro::{AFastDeserialize, AFastSerialize};

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
