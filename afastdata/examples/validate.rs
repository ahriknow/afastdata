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
        "${field} must be greater than or equal to 10 and less than or equal to 100"
    ))]
    b: Option<String>,

    #[afast(func("v"))]
    c: i32,

    #[afast(of([1, 2, 3], 0, "${field} must be one of [1, 2, 3]"))]
    d: i64,
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

fn main() {
    let a = A {
        a: 100,
        b: Some("1234567890".to_string()),
        c: 50,
        d: 1,
    };
    let a_bytes = a.to_bytes();
    let a_copy = A::from_bytes(&a_bytes).unwrap();
    println!("Original: {:?}, Round-trip: {:?}", a, a_copy);
}
