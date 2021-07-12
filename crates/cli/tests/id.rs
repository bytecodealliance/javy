use anyhow::Result;
use proptest::prelude::*;
use serde::Serialize;

mod runner;
mod convert;

use crate::runner::Runner;

#[derive(Clone, Debug, Serialize)]
enum V {
    Null(()),
    Int(i32),
    Float(f64),
    Boolean(bool),
    String(String),
}

fn value() -> impl Strategy<Value = V> {
    prop_oneof![
        Just(V::Null(())),
        any::<i32>().prop_map(V::Int),
        any::<f64>().prop_map(V::Float),
        any::<bool>().prop_map(V::Boolean),
        any::<String>().prop_map(V::String),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_time: 10,
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_i32(s in any::<i32>()) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        prop_assert_eq!(Some(s as i64), output.as_i64());
    }

    #[test]
    fn test_u16(s in any::<u16>()) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        prop_assert_eq!(Some(s as i64), output.as_i64())
    }

    #[test]
    fn test_u32(s in any::<u32>()) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        // Should be able to cast to f64
        prop_assert_eq!(Some(s as f64), output.as_f64());
    }

    #[test]
    fn test_f32(s in any::<f32>()) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        // Should be able to cast to f64
        prop_assert_eq!(Some(s as f64), output.as_f64())
    }

    #[test]
    fn test_f64(s in any::<f64>()) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        // Should be able to cast to f64
        prop_assert_eq!(Some(s), output.as_f64());
    }

    #[test]
    fn test_string(s in any::<String>()) {
        println!("{:?}", s);
        let bytes: Vec<u8> = convert::to_rmp_bytes(&s);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        prop_assert_eq!(Some(s.as_str()), output.as_str());
    }

    #[test]
    fn test_array_primitives(v in proptest::collection::vec(value(), 1..100)) {
        let bytes: Vec<u8> = convert::to_rmp_bytes(&v);

        let result = Runner::new().unwrap()
            .define_imports().unwrap()
            .exec(bytes).unwrap();

        let output = convert::to_rmpv(&result);

        prop_assert!(output.is_array());
        prop_assert_eq!(output.as_array().unwrap().len(), v.len());
    }
}

#[test]
fn test_null() -> Result<()> {
    let bytes: Vec<u8> = convert::to_rmp_bytes(());

    let result = Runner::new()?
        .define_imports()?
        .exec(bytes)?;

    let output = convert::to_rmpv(&result);

    assert!(output.is_nil());

    Ok(())
}

#[test]
fn test_bool() -> Result<()> {
    let input = vec![true, false];
    let mut runner = Runner::new()?;
    runner.define_imports()?;

    for v in input {
        let bytes: Vec<u8> = convert::to_rmp_bytes(v);
        let result = runner.exec(bytes)?;
        let output = convert::to_rmpv(&result);

        assert_eq!(v, output.as_bool().unwrap());
    }

    Ok(())
}
