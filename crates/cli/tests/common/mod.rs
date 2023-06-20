use anyhow::Result;
use std::path::{Path, PathBuf};
use wasmparser::ProducersSectionReader;
use wasmtime::{Engine, Module};

// Allows dead code b/c each integration test suite is considered its own
// application and this function is used by 2 of 3 suites.
#[allow(dead_code)]
pub fn create_quickjs_provider_module(engine: &Engine) -> Result<Module> {
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.pop();
    lib_path.pop();
    lib_path = lib_path.join(
        Path::new("target")
            .join("wasm32-wasi")
            .join("release")
            .join("javy_quickjs_provider_wizened.wasm"),
    );
    Module::from_file(engine, lib_path)
}

// Allows dead code b/c each integration test suite is considered its own
// application and this function is used by 2 of 3 suites.
#[allow(dead_code)]
pub fn assert_producers_section_is_correct(wasm: &[u8]) -> Result<()> {
    let producers_section = wasmparser::Parser::new(0)
        .parse_all(wasm)
        .find_map(|payload| match payload {
            Ok(wasmparser::Payload::CustomSection(c)) if c.name() == "producers" => {
                Some(ProducersSectionReader::new(c.data(), c.data_offset()).unwrap())
            }
            _ => None,
        })
        .unwrap();
    let fields = producers_section
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(2, fields.len());

    let language_field = &fields[0];
    assert_eq!("language", language_field.name);
    assert_eq!(1, language_field.values.count());
    let language_value = language_field.values.clone().into_iter().next().unwrap()?;
    assert_eq!("JavaScript", language_value.name);
    assert_eq!("ES2020", language_value.version);

    let processed_by_field = &fields[1];
    assert_eq!("processed-by", processed_by_field.name);
    assert_eq!(1, processed_by_field.values.count());
    let processed_by_value = processed_by_field
        .values
        .clone()
        .into_iter()
        .next()
        .unwrap()?;
    assert_eq!("Javy", processed_by_value.name);
    Ok(())
}
