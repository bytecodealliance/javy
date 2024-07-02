use anyhow::bail;
/// Macros for testing Javy.
///
/// Helper macros to define Test262 tests or tests that exercise different
/// configuration combinations.
///
/// Currently only defining Test262 tests for JSON is supported.
///
/// Usage
///
/// ```rust
/// t262!(path = "path/to/262/directory")
/// ```
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, Ident, LitStr, Result};

struct Config262 {
    root: PathBuf,
}

impl Config262 {
    fn validate(&self) -> anyhow::Result<()> {
        let path: Box<Path> = self.root.clone().into();
        if path.is_dir() {
            Ok(())
        } else {
            bail!("Invalid path")
        }
    }
}

impl Default for Config262 {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
        }
    }
}

#[proc_macro]
pub fn t262(stream: TokenStream) -> TokenStream {
    let mut config = Config262::default();

    let config_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            let lit: Option<LitStr> = Some(meta.value()?.parse()?);

            if let Some(s) = lit {
                config.root = PathBuf::from(s.clone().value());
            } else {
                return Err(meta.error("Expected String literal"));
            }
            config.validate().map_err(|e| meta.error(e))
        } else {
            Err(meta.error("Unsupported property"))
        }
    });

    parse_macro_input!(stream with config_parser);

    match expand(&config) {
        Ok(tok) => tok,
        Err(e) => e.into_compile_error().into(),
    }
}

// Should this test be ignored?
fn ignore(test_name: &str) -> bool {
    [
        // A bit unfortunate, currently simd-json returns `0` for `'-0'`;
        // I think this is a bug in simd-json itself.
        "test_parse_text_negative_zero",
        // Realms are not supported by QuickJS
        "test_stringify_replacer_array_proxy_revoked_realm",
        "test_stringify_value_bigint_cross_realm",
        // TODO
        // Currently the conversion between non-utf8 string encodings is lossy.
        // There's probably a way to improve the interop.
        "test_stringify_value_string_escape_unicode",
    ]
    .contains(&test_name)
}

fn expand(config: &Config262) -> Result<TokenStream> {
    let harness = config.root.join("harness");
    let harness_str = harness.into_os_string().into_string().unwrap();
    let json_parse = config
        .root
        .join("test")
        .join("built-ins")
        .join("JSON")
        .join("parse");

    let json_stringify = config
        .root
        .join("test")
        .join("built-ins")
        .join("JSON")
        .join("stringify");

    let parse_tests = gen_tests(&json_parse, &harness_str, "parse");
    let stringify_tests = gen_tests(&json_stringify, &harness_str, "stringify");

    Ok(quote! {
        #parse_tests
        #stringify_tests
    }
    .into())
}

fn gen_tests(
    dir: &PathBuf,
    harness_str: &String,
    prefix: &'static str,
) -> proc_macro2::TokenStream {
    let parse_dir = std::fs::read_dir(dir).expect("parse directory to be available");
    let spec = parse_dir.filter_map(|e| e.ok()).map(move |entry| {
            let path = entry.path();
            let path_str = path.clone().into_os_string().into_string().unwrap();
            let name = path.file_stem().unwrap().to_str().unwrap();
            let name = name.replace('.', "_");
            let name = name.replace('-', "_");
            let test_name = Ident::new(&format!("test_{}_{}", prefix, name), Span::call_site());
            let ignore = ignore(&test_name.to_string());

            let attrs = if ignore {
                quote! {
                    #[ignore]
                }
            } else {
                quote! {}
            };

            quote! {
                #[test]
                #attrs
                #[allow(non_snake_case)]
                fn #test_name() {
                    let mut config = ::javy::Config::default();
                    config
                        .override_json_parse_and_stringify(true);
                    let runtime = ::javy::Runtime::new(config).expect("runtime to be created");
                    let harness_path = ::std::path::PathBuf::from(#harness_str);

                    let helpers = vec![
                        harness_path.join("sta.js"),
                        harness_path.join("assert.js"),
                        harness_path.join("compareArray.js"),
                        harness_path.join("propertyHelper.js"),
                        harness_path.join("isConstructor.js")
                    ];
                    runtime.context().with(|this| {
                        for helper in helpers {
                            let helper_contents = ::std::fs::read(helper).expect("helper path to exist");
                            let r: ::javy::quickjs::Result<()> = this.eval_with_options(helper_contents, ::javy::quickjs::context::EvalOptions::default()).expect("helper evaluation to succeed");
                            assert!(r.is_ok());
                        }

                        let test_contents = ::std::fs::read(&#path_str).expect("test file contents to be available");
                        let r: ::javy::quickjs::Result<()> = this.eval_with_options(test_contents, ::javy::quickjs::context::EvalOptions::default());
                        assert!(r.is_ok(), "{}", ::javy::val_to_string(&this, this.catch()).unwrap());
                    });

                }
            }
        });

    quote! {
        #(#spec)*
    }
}
