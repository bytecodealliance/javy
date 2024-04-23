// use crate::quickjs::JSContextRef;
use super::from_js_error;
use anyhow::{bail, Result};
use rquickjs::{Context, Module, Runtime as QRuntime};
use std::mem::ManuallyDrop;

use crate::Config;

/// A JavaScript Runtime.
///
/// Javy's [`Runtime`] holds a [`rquickjs::Runtime`] and [`rquickjs::Context`],
/// and provides accessors these two propoerties which enable working with
/// [`rquickjs`] APIs.
pub struct Runtime {
    /// The QuickJS context.
    // We use `ManuallyDrop` to avoid incurring in the cost of dropping the
    // `rquickjs::Context` and its associated objects, which takes a substantial
    // amount of time.
    //
    // This assumes that Javy is used for short-lived programs where the host
    // will collect the instance's memory when execution ends, making these
    // drops unnecessary.
    //
    // This might not be suitable for all use-cases, so we'll make this
    // behaviour configurable.
    context: ManuallyDrop<Context>,
    /// The inner QuickJS runtime representation.
    // Read above on the usage of `ManuallyDrop`.
    inner: ManuallyDrop<QRuntime>,
}

impl Runtime {
    /// Creates a new [Runtime].
    pub fn new(_config: Config) -> Result<Self> {
        let rt = ManuallyDrop::new(QRuntime::new()?);

        // See comment above about configuring GC behaviour.
        rt.set_gc_threshold(usize::MAX);
        let context = ManuallyDrop::new(Context::full(&rt)?);
        Ok(Self { inner: rt, context })
    }

    /// A reference to the inner [Context].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Resolves all the pending jobs in the queue.
    pub fn resolve_pending_jobs(&self) -> Result<()> {
        if self.inner.is_job_pending() {
            loop {
                let result = self.inner.execute_pending_job();
                if let Ok(false) = result {
                    break;
                }

                if let Err(e) = result {
                    bail!("{e}")
                }
            }
        }

        Ok(())
    }

    /// Returns true if there are pending jobs in the queue.
    pub fn has_pending_jobs(&self) -> bool {
        self.inner.is_job_pending()
    }

    /// Compiles the given module to bytecode.
    pub fn compile_to_bytecode(&self, name: &str, contents: &str) -> Result<Vec<u8>> {
        self.context()
            .with(|this| {
                unsafe { Module::unsafe_declare(this.clone(), name, contents) }?.write_object_le()
            })
            .map_err(|e| self.context().with(|cx| from_js_error(cx.clone(), e)))
    }
}

impl Default for Runtime {
    /// Returns a [`Runtime`] with a default configuration.
    ///
    /// # Panics
    /// This function panics if there is an error setting up the runtime.
    fn default() -> Self {
        Self::new(Config::default()).unwrap()
    }
}
