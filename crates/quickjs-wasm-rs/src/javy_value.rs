// use std::{collections::HashMap};

// Should this type be in a completely separate crate if we plan to have multiple JS engines? 
// That way the spidermonkey engine can also use to serialize to their internal types
pub enum JavyValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32), // do we need to support i8..i64?
    Float(f64),
    String(String),
    // Array(Vec<JavyValue>),
    // Object(HashMap<String, JavyValue>),    
}
