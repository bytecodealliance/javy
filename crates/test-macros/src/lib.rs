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
use std::{env, path::PathBuf};
use syn::{meta::ParseNestedMeta, parse_macro_input, Ident, LitBool, LitStr, Result, ReturnType};

struct Config262 {
    root: PathBuf,
}

impl Config262 {
    fn validate(&self) -> anyhow::Result<()> {
        if PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join(&self.root)
            .is_dir()
        {
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

    match expand_262(&config) {
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

fn expand_262(config: &Config262) -> Result<TokenStream> {
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
    let package_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let parse_dir =
        std::fs::read_dir(package_path.join(dir)).expect("parse directory to be available");
    let spec = parse_dir.filter_map(|e| e.ok()).map(move |entry| {
            let path = entry.path();
            let path = path.strip_prefix(&package_path).unwrap();
            let path_str = path.to_str().unwrap();
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
                        .simd_json_builtins(true);
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

struct CliTestConfig {
    /// Root directory to load test scripts from, relative to the crate's
    /// directory (i.e., `CARGO_MANIFEST_DIR`)
    scripts_root: String,
    /// Which commands to generate the test for. It can be either `compile` or
    /// `build`.
    commands: Vec<Ident>,
    /// Tests Javy's dynamic linking capabilities.
    dynamic: bool,
}

impl CliTestConfig {
    fn commands_from(&mut self, meta: &ParseNestedMeta) -> Result<()> {
        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("not") {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Compile") || meta.path.is_ident("Build") {
                        let id = meta.path.require_ident()?.clone();
                        self.commands.retain(|s| *s != id);
                        Ok(())
                    } else {
                        Err(meta.error("Unknown command"))
                    }
                })
            } else {
                Err(meta.error("Unknown identifier"))
            }
        })?;
        Ok(())
    }

    fn root_from(&mut self, meta: &ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("root") {
            let val = meta.value()?;
            let val: LitStr = val.parse()?;
            self.scripts_root = val.value();
            Ok(())
        } else {
            Err(meta.error("Unknown value"))
        }
    }

    fn dynamic_from(&mut self, meta: &ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("dyn") {
            let val = meta.value()?;
            let val: LitBool = val.parse()?;
            self.dynamic = val.value();
            Ok(())
        } else {
            Err(meta.error("Unknown value"))
        }
    }
}

impl Default for CliTestConfig {
    fn default() -> Self {
        Self {
            scripts_root: String::from("tests/sample-scripts"),
            commands: vec![
                Ident::new("Compile", Span::call_site()),
                Ident::new("Build", Span::call_site()),
            ],
            dynamic: false,
        }
    }
}

#[proc_macro_attribute]
pub fn javy_cli_test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut config = CliTestConfig::default();
    let config_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("commands") {
            config.commands_from(&meta)
        } else if meta.path.is_ident("root") {
            config.root_from(&meta)
        } else if meta.path.is_ident("dyn") {
            config.dynamic_from(&meta)
        } else {
            Err(meta.error("Unsupported attributes"))
        }
    });

    parse_macro_input!(attrs with config_parser);

    match expand_cli_tests(&config, parse_macro_input!(item as syn::ItemFn)) {
        Ok(tok) => tok,
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand_cli_tests(test_config: &CliTestConfig, func: syn::ItemFn) -> Result<TokenStream> {
    let mut tests = vec![quote! { #func }];
    let attrs = &func.attrs;

    for ident in &test_config.commands {
        let command_name = ident.to_string();
        let func_name = &func.sig.ident;
        let ret = match &func.sig.output {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, ty) => quote! { -> #ty },
        };
        let test_name = Ident::new(
            &format!("{}_{}", command_name.to_lowercase(), func_name),
            func_name.span(),
        );

        let preload_setup = if test_config.dynamic {
            // The compile commmand will remain frozen until it is deleted.
            // Until then we test with a frozen artifact downloaded from the
            // releases.
            if command_name == "Compile" {
                quote! {
                    let plugin = javy_runner::Plugin::V2;
                    builder.preload(
                        plugin.namespace().into(),
                        plugin.path(),
                    );
                    builder.plugin(plugin);
                }
            } else {
                quote! {
                    let plugin = javy_runner::Plugin::DefaultAsUser;
                    builder.preload(plugin.namespace().into(), plugin.path());
                    builder.plugin(plugin);
                }
            }
        } else {
            quote! {}
        };

        let root = test_config.scripts_root.clone();

        let tok = quote! {
            #[test]
            #(#attrs)*
            fn #test_name() #ret {
                let mut builder = javy_runner::Builder::default();
                builder.command(javy_runner::JavyCommand::#ident);
                builder.root(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(#root));
                builder.bin(env!("CARGO_BIN_EXE_javy"));

                #preload_setup

                #func_name(&mut builder)
            }
        };

        tests.push(tok);
    }
    Ok(quote! {
        #(#tests)*
    }
    .into())
}
