/// Higher-level representation of JavaScript.
///
/// This is intended to be used to derive different representations of source
/// code. For example, as a byte array, a string, QuickJS bytecode, compressed
/// bytes, or attributes of the source code like what it exports.
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    path::Path,
    rc::Rc,
};

use anyhow::{anyhow, bail, Context, Result};
use brotli::enc::{self, BrotliEncoderParams};
use swc_core::{
    common::{FileName, SourceMap},
    ecma::{
        ast::{
            Decl, EsVersion, ExportDecl, ExportSpecifier, Module, ModuleDecl, ModuleExportName,
            ModuleItem, Stmt,
        },
        parser::{self, EsSyntax, Syntax},
    },
};

use crate::plugin::Plugin;

/// JS source code.
#[derive(Clone, Debug)]
pub struct JS {
    source_code: Rc<String>,
}

impl JS {
    /// Create [`JS`] from a string containing JS source code.
    pub fn from_string(source_code: String) -> JS {
        JS {
            source_code: Rc::new(source_code),
        }
    }

    /// Create [`JS`] from a file containing JS.
    pub fn from_file(path: &Path) -> Result<JS> {
        let mut input_file = File::open(path)
            .with_context(|| format!("Failed to open input file {}", path.display()))?;
        let mut contents: Vec<u8> = vec![];
        input_file.read_to_end(&mut contents)?;
        Ok(Self::from_string(String::from_utf8(contents)?))
    }

    /// Get source code as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.source_code.as_bytes()
    }

    /// Compiles a JavaScript source to bytecode using a QuickJS plugin.
    pub(crate) fn compile(&self, plugin: &Plugin) -> Result<Vec<u8>> {
        plugin.compile_source(self.source_code.as_bytes())
    }

    /// Get Brotli compressed JS source code as bytes.
    pub(crate) fn compress(&self) -> Result<Vec<u8>> {
        let mut compressed_source_code: Vec<u8> = vec![];
        enc::BrotliCompress(
            &mut Cursor::new(&self.source_code.as_bytes()),
            &mut compressed_source_code,
            &BrotliEncoderParams {
                quality: 11,
                ..Default::default()
            },
        )?;
        Ok(compressed_source_code)
    }

    /// Get the exports from a JS instance.
    pub(crate) fn exports(&self) -> Result<Vec<String>> {
        let module = self.parse_module()?;

        // function foo() ...
        let mut functions = HashMap::new();
        // export { foo, bar as baz }
        let mut named_exports = vec![];
        // export function foo() ...
        let mut exported_functions = vec![];
        for item in module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Fn(f),
                    ..
                })) => {
                    if !f.function.params.is_empty() {
                        bail!("Exported functions with parameters are not supported");
                    }
                    if f.function.is_generator {
                        bail!("Exported generators are not supported");
                    }
                    exported_functions.push(f.ident.sym);
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) => {
                    for specifier in e.specifiers {
                        if let ExportSpecifier::Named(n) = specifier {
                            let orig = match n.orig {
                                ModuleExportName::Ident(i) => i.sym,
                                ModuleExportName::Str(s) => s.value,
                            };
                            let exported_name = n.exported.map(|e| match e {
                                ModuleExportName::Ident(i) => i.sym,
                                ModuleExportName::Str(s) => s.value,
                            });
                            named_exports.push((orig, exported_name));
                        }
                    }
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(e)) if e.decl.is_fn_expr() => {
                    exported_functions.push("default".into())
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(e)) if e.expr.is_arrow() => {
                    exported_functions.push("default".into())
                }
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(f))) => {
                    functions.insert(
                        f.ident.sym,
                        (f.function.params.is_empty(), f.function.is_generator),
                    );
                }
                _ => continue,
            }
        }

        let mut named_exported_functions = named_exports
            .into_iter()
            .filter_map(|(orig, exported)| {
                if let Some((no_params, is_generator)) = functions.get(&orig) {
                    if !no_params {
                        Some(Err(anyhow!(
                            "Exported functions with parameters are not supported"
                        )))
                    } else if *is_generator {
                        Some(Err(anyhow!("Exported generators are not supported")))
                    } else {
                        Some(Ok(exported.unwrap_or(orig)))
                    }
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        exported_functions.append(&mut named_exported_functions);
        Ok(exported_functions
            .into_iter()
            .map(|f| f.to_string())
            .collect())
    }

    fn parse_module(&self) -> Result<Module> {
        let source_map: SourceMap = Default::default();
        let file = source_map.new_source_file_from(FileName::Anon.into(), self.source_code.clone());
        let mut errors = vec![];
        parser::parse_file_as_module(
            &file,
            Syntax::Es(EsSyntax::default()),
            EsVersion::Es2020,
            None,
            &mut errors,
        )
        .map_err(|e| anyhow!(e.into_kind().msg()))
        .with_context(|| "Invalid JavaScript")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::js::JS;

    #[test]
    fn parse_no_exports() -> Result<()> {
        let exports = parse("function foo() {}")?;
        assert_eq!(Vec::<&str>::default(), exports);
        Ok(())
    }

    #[test]
    fn parse_invalid_js() -> Result<()> {
        let res = parse("fun foo() {}");
        assert_eq!("Invalid JavaScript", res.err().unwrap().to_string());
        Ok(())
    }

    #[test]
    fn parse_one_func_export() -> Result<()> {
        let exports = parse("export function foo() {}")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_func_export_with_parameter() -> Result<()> {
        let res = parse("export function foo(bar) {}");
        assert_eq!(
            "Exported functions with parameters are not supported",
            res.err().unwrap().to_string()
        );
        Ok(())
    }

    #[test]
    fn parse_generator_export() -> Result<()> {
        let res = parse("export function *foo() {}");
        assert_eq!(
            "Exported generators are not supported",
            res.err().unwrap().to_string()
        );
        Ok(())
    }

    #[test]
    fn parse_two_func_exports() -> Result<()> {
        let exports = parse("export function foo() {}; export function bar() {};")?;
        assert_eq!(vec!["foo", "bar"], exports);
        Ok(())
    }

    #[test]
    fn parse_const_export() -> Result<()> {
        let exports = parse("export const x = 1;")?;
        let expected_exports: Vec<&str> = vec![];
        assert_eq!(expected_exports, exports);
        Ok(())
    }

    #[test]
    fn parse_const_export_and_func_export() -> Result<()> {
        let exports = parse("export const x = 1; export function foo() {}")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_named_func_export() -> Result<()> {
        let exports = parse("function foo() {}; export { foo };")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_named_func_export_with_arg() -> Result<()> {
        let res = parse("function foo(bar) {}; export { foo };");
        assert_eq!(
            "Exported functions with parameters are not supported",
            res.err().unwrap().to_string()
        );
        Ok(())
    }

    #[test]
    fn parse_funcs_with_args() -> Result<()> {
        let exports = parse("function foo(bar) {}")?;
        assert_eq!(Vec::<&str>::default(), exports);
        Ok(())
    }

    #[test]
    fn parse_named_func_export_and_const_export() -> Result<()> {
        let exports = parse("function foo() {}; const bar = 1; export { foo, bar };")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_func_export_and_named_func_export() -> Result<()> {
        let exports = parse("export function foo() {}; function bar() {}; export { bar };")?;
        assert_eq!(vec!["foo", "bar"], exports);
        Ok(())
    }

    #[test]
    fn parse_renamed_func_export() -> Result<()> {
        let exports = parse("function foo() {}; export { foo as bar };")?;
        assert_eq!(vec!["bar"], exports);
        Ok(())
    }

    #[test]
    fn parse_hoisted_func_export() -> Result<()> {
        let exports = parse("export { foo }; function foo() {}")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_renamed_hosted_func_export() -> Result<()> {
        let exports = parse("export { foo as bar }; function foo() {}")?;
        assert_eq!(vec!["bar"], exports);
        Ok(())
    }

    #[test]
    fn parse_hoisted_exports_with_func_and_const() -> Result<()> {
        let exports = parse("export { foo, bar }; function foo() {}; const bar = 1;")?;
        assert_eq!(vec!["foo"], exports);
        Ok(())
    }

    #[test]
    fn parse_default_arrow_export() -> Result<()> {
        let exports = parse("export default () => {}")?;
        assert_eq!(vec!["default"], exports);
        Ok(())
    }

    #[test]
    fn parse_default_function_export() -> Result<()> {
        let exports = parse("export default function() {}")?;
        assert_eq!(vec!["default"], exports);
        Ok(())
    }

    fn parse(js: &str) -> Result<Vec<String>> {
        JS::from_string(js.to_string()).exports()
    }
}
