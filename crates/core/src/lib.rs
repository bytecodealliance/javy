const fn byte_string_len(s: &[u8]) -> usize {
    s.len()
}

#[link_section = "import_namespace"]
pub static IMPORT_NAMESPACE: [u8; byte_string_len(b"javy_quickjs_provider_v3")] =
    *b"javy_quickjs_provider_v3";

#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    javy::exported_fns::initialize_runtime(None, |runtime| runtime);
}
