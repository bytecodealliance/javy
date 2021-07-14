use serde::Serialize;

pub fn to_rmp_bytes<T>(input: T) -> Vec<u8>
where T: Serialize {
    rmp_serde::to_vec(&input).expect("Couldn't serialize to msgpack")
}

pub fn to_rmpv(bytes: &[u8]) -> rmpv::Value {
    let val: rmpv::Value = rmp_serde::from_slice(bytes).expect("Couldn't convert bytes to mesgpack value");
    val
}

