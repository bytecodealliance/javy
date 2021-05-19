
use quickjs_sys as q;

#[no_mangle]
pub fn hi() {
    print!("Hi")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
