/// Create a custom section named `import_namespace` with the contents of the
/// string argument.
#[macro_export]
macro_rules! import_namespace {
    ($str:literal) => {
        const IMPORT_NAMESPACE_BYTES: &[u8] = $str.as_bytes();

        #[link_section = "import_namespace"]
        pub static IMPORT_NAMESPACE: [u8; IMPORT_NAMESPACE_BYTES.len()] = {
            let mut arr = [0u8; IMPORT_NAMESPACE_BYTES.len()];
            let mut i = 0;
            while i < IMPORT_NAMESPACE_BYTES.len() {
                arr[i] = IMPORT_NAMESPACE_BYTES[i];
                i += 1;
            }
            arr
        };
    };
}
