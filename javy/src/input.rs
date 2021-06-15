use quickjs_sys as q;
use inflector::Inflector;

pub fn prepare(context: &crate::Context, bytes: &[u8]) -> q::JSValue {
    let mut value: rmpv::Value = rmp_serde::from_read_ref(&bytes).unwrap();
    process_value(&mut value);
    let json = serde_json::to_string(&value).unwrap();


    context.serialize_string(&json)
}

// TODO: Find a more idiomatic way to do this throug serde. I tried applying a
// rename directive when deserializing but it didn't work. Not sure why; performance with this
// approach is acceptable so I'll revisit this later.
fn process_value(val: &mut rmpv::Value) {
    match val {
        rmpv::Value::Map(vec) => process_map(vec.as_mut()),
        rmpv::Value::Array(vec) => process_array(vec.as_mut()),
        _ => (),
    }
}

fn process_map(map: &mut Vec<(rmpv::Value, rmpv::Value)>) {
    for (k, v) in map.iter_mut() {
        let string = Inflector::to_camel_case(&k.to_string());
        *k = rmpv::Value::from(string);
        process_value(v);
    }
}

fn process_array(array: &mut Vec<rmpv::Value>) {
    for v in array {
        process_value(v);
    }
}
