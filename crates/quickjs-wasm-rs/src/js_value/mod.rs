use std::{collections::HashMap, fmt};

pub mod qjs_convert;
mod to_js_value;
mod try_from_js_value;

#[derive(Debug, PartialEq, Clone)]
pub enum JSValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Array(Vec<JSValue>),
    ArrayBuffer(Vec<u8>),
    Object(HashMap<String, JSValue>),
}

impl JSValue {
    pub fn from_vec<T: Into<JSValue>>(v: Vec<T>) -> JSValue {
        let js_arr: Vec<JSValue> = v.into_iter().map(|elem| elem.into()).collect();
        JSValue::Array(js_arr)
    }

    pub fn from_hashmap<T: Into<JSValue>>(hm: HashMap<&str, T>) -> JSValue {
        let js_obj: HashMap<String, JSValue> = hm
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();
        JSValue::Object(js_obj)
    }
}

// Used http://numcalc.com/ to determine the default display format for each type
impl fmt::Display for JSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSValue::Undefined => write!(f, "undefined"),
            JSValue::Null => write!(f, "null"),
            JSValue::Bool(b) => write!(f, "{}", b),
            JSValue::Int(i) => write!(f, "{}", i),
            JSValue::Float(n) => write!(f, "{}", n),
            JSValue::String(s) => write!(f, "{}", s),
            JSValue::ArrayBuffer(_) => write!(f, "{{  }}"),
            JSValue::Array(arr) => {
                write!(
                    f,
                    "{}",
                    arr.iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            JSValue::Object(_) => write!(f, "[object Object]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_between_bool() {
        let js_value: JSValue = true.into();
        assert_eq!("true", js_value.to_string());

        let result: bool = js_value.try_into().unwrap();
        assert!(result);
    }

    #[test]
    fn test_conversion_between_i32() {
        let js_value: JSValue = 2.into();
        assert_eq!("2", js_value.to_string());

        let result: i32 = js_value.try_into().unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_conversion_between_usize() {
        let i: usize = 1;
        let js_value: JSValue = i.into();
        assert_eq!("1", js_value.to_string());

        let result: usize = js_value.try_into().unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_conversion_between_f64() {
        let js_value: JSValue = 2.3.into();
        assert_eq!("2.3", js_value.to_string());

        let result: f64 = js_value.try_into().unwrap();
        assert_eq!(result, 2.3);
    }

    #[test]
    fn test_conversion_between_str_and_string() {
        let js_value: JSValue = "hello".into();
        assert_eq!("hello", js_value.to_string());

        let js_value: JSValue = "hello".to_string().into();
        assert_eq!("hello", js_value.to_string());

        let result: String = js_value.try_into().unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_conversion_between_vec() {
        let js_value: JSValue = JSValue::from_vec(vec![1, 2]);
        assert_eq!("1,2", js_value.to_string());

        let result: Vec<JSValue> = js_value.try_into().unwrap();
        assert_eq!(result, vec![JSValue::Int(1), JSValue::Int(2)]);

        let js_value: JSValue = JSValue::from_vec(vec!["h", "w"]);
        assert_eq!("h,w", js_value.to_string());

        let result: Vec<JSValue> = js_value.try_into().unwrap();
        assert_eq!(
            result,
            vec![
                JSValue::String("h".to_string()),
                JSValue::String("w".to_string())
            ]
        );
    }

    #[test]
    fn test_conversion_between_bytes() {
        let bytes = "hi".as_bytes();
        let js_value: JSValue = bytes.into();
        assert_eq!("{  }", js_value.to_string());

        let result: Vec<u8> = js_value.try_into().unwrap();
        assert_eq!(result, bytes.to_vec());
    }

    #[test]
    fn test_conversion_between_hashmap() {
        let map = HashMap::from([("a", 1), ("b", 2)]);
        let js_value = JSValue::from_hashmap(map);
        assert_eq!("[object Object]", js_value.to_string());

        let result: HashMap<String, JSValue> = js_value.try_into().unwrap();
        assert_eq!(
            result,
            HashMap::from([
                ("a".to_string(), JSValue::Int(1)),
                ("b".to_string(), JSValue::Int(2)),
            ])
        );
    }
}
