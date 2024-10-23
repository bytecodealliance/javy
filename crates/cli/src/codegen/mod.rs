//! WebAssembly Code Generation for JavaScript
//!
//! This module provides all the functionality to emit Wasm modules for
//! a particular JavaScript program.
//!
//! Javy supports two main code generation paths:
//!
//! 1. Static code generation
//! 2. Dynamic code generation
//!
//!
//! ## Static code generation
//!
//! A single unit of code is generated, which is a Wasm module consisting of the
//! bytecode representation of a given JavaScript program and the code for
//! a particular version of the QuickJS engine compiled to Wasm.
//!
//! The generated Wasm module is self contained and the bytecode version matches
//! the exact requirements of the embedded QuickJs engine.
//!
//! ## Dynamic code generation
//!
//! A single unit of code is generated, which is a Wasm module consisting of the
//! bytecode representation of a given JavaScript program. The JavaScript
//! bytecode is stored as part of the data section of the module which also
//! contains instructions to execute that bytecode through dynamic linking
//! at runtime.
//!
//! Dynamic code generation requires a provider module to be used and linked
//! against at runtime in order to execute the JavaScript bytecode. This
//! operation involves carefully ensuring that a given provider version matches
//! the provider version of the imports requested by the generated Wasm module
//! as well as ensuring that any features available in the provider match the
//! features requsted by the JavaScript bytecode.

mod builder;
pub(crate) use builder::*;

mod dynamic;
pub(crate) use dynamic::*;

mod r#static;
pub(crate) use r#static::*;

mod transform;

mod exports;
pub(crate) use exports::*;

use crate::JS;

pub(crate) enum CodeGenType {
    /// Static code generation.
    Static,
    /// Dynamic code generation.
    Dynamic,
}

/// Code generator trait to abstract the multiple JS to Wasm code generation
/// paths.
pub(crate) trait CodeGen {
    /// Generate Wasm from a given JS source.
    fn generate(&mut self, source: &JS) -> anyhow::Result<Vec<u8>>;
}
