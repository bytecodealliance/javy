use std::{collections::HashMap, fmt};

pub mod qjs_convert;

mod to_js_value;
mod try_from_js_value;

/// A safe and high level representation of a JavaScript value.
///
/// This enum implements `From` and `TryFrom` for many types, so it can be used to convert between Rust and JavaScript types.
///
/// # Example
///
/// ```
/// // Convert a &str to a JSValue::String
/// let js_value: JSValue = "hello".into();
/// assert_eq!("hello", js_value.to_string());
///
/// // Convert a JSValue::String to a String
/// let result: String = js_value.try_into().unwrap();
/// assert_eq!("hello", result);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum JSValue {
    /// Represents the JavaScript `undefined` value
    Undefined,
    /// Represents the JavaScript `null` value
    Null,
    /// Represents a JavaScript boolean value
    Bool(bool),
    /// Represents a JavaScript integer
    Int(i32),
    /// Represents a JavaScript floating-point number
    Float(f64),
    /// Represents a JavaScript string value
    String(String),
    /// Represents a JavaScript array of `JSValue`s
    Array(Vec<JSValue>),
    /// Represents a JavaScript ArrayBuffer of bytes
    ArrayBuffer(Vec<u8>),
    /// Represents a JavaScript object, with string keys and `JSValue` values
    Object(HashMap<String, JSValue>),
}

impl JSValue {
    /// Constructs a `JSValue::Array` variant from a vec of items that can be converted to `JSValue`.
    ///
    /// # Arguments
    ///
    /// * `v` - A vec of items to be converted to `JSValue::Array`
    ///
    /// # Example
    ///
    /// ```
    /// let vec = vec![1, 2, 3];
    /// let js_arr = JSValue::from_vec(vec);
    /// ```
    pub fn from_vec<T: Into<JSValue>>(v: Vec<T>) -> JSValue {
        let js_arr: Vec<JSValue> = v.into_iter().map(|elem| elem.into()).collect();
        JSValue::Array(js_arr)
    }

    /// Constructs a `JSValue::Object` variant from a HashMap of key-value pairs that can be converted to `JSValue`.
    ///
    /// # Arguments
    ///
    /// * `hm` - A HashMap of key-value pairs to be converted to `JSValue::Object`
    ///
    /// # Example
    ///
    /// ```
    /// let mut hashmap = std::collections::HashMap::from([
    ///   ("first_name", "John"),
    ///   ("last_name", "Smith"),
    /// ]);
    ///
    /// let js_obj = JSValue::from_hashmap(hashmap);
    /// ```
    pub fn from_hashmap<T: Into<JSValue>>(hm: HashMap<&str, T>) -> JSValue {
        let js_obj: HashMap<String, JSValue> = hm
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();
        JSValue::Object(js_obj)
    }
}

/// The implementation matches the default JavaScript display format for each value.
///
/// Used <http://numcalc.com/> to determine the default display format for each type.
impl fmt::Display for JSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSValue::Undefined => write!(f, "undefined"),
            JSValue::Null => write!(f, "null"),
            JSValue::Bool(b) => write!(f, "{}", b),
            JSValue::Int(i) => write!(f, "{}", i),
            JSValue::Float(n) => write!(f, "{}", n),
            JSValue::String(s) => write!(f, "{}", s),
            JSValue::ArrayBuffer(_) => write!(f, "[object ArrayBuffer]"),
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
        assert_eq!("[object ArrayBuffer]", js_value.to_string());

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
