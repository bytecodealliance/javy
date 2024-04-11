// use crate::quickjs::JSContextRef;
use anyhow::{bail, Result};
use rquickjs::{Context, Runtime as QRuntime};
use std::mem::ManuallyDrop;

use crate::Config;

// TODO: Update documentation.
/// A JavaScript Runtime.
///
/// Provides a [`Self::context()`] method for working with the underlying [`JSContextRef`].
///
/// ## Examples
///
/// ```
/// # use anyhow::anyhow;
/// # use javy::{quickjs::JSValue, Runtime};
/// let runtime = Runtime::default();
/// let context = runtime.context();
/// context
///     .global_object()
///     .unwrap()
///     .set_property(
///         "print",
///         context
///             .wrap_callback(move |_ctx, _this, args| {
///                 let str = args
///                     .first()
///                     .ok_or(anyhow!("Need to pass an argument"))?
///                     .to_string();
///                 println!("{str}");
///                 Ok(JSValue::Undefined)
///             })
///             .unwrap(),
///     )
///     .unwrap();
/// context.eval_global("hello.js", "print('hello!');").unwrap();
/// ```
pub struct Runtime {
    /// The QuickJS context.
    context: Context,
    /// The inner QuickJS runtime representation.
    // TODO: Document why `ManuallyDrop`.
    inner: ManuallyDrop<QRuntime>,
}

impl Runtime {
    /// Creates a new [Runtime].
    pub fn new(_config: Config) -> Result<Self> {
        let rt = ManuallyDrop::new(QRuntime::new()?);

        // TODO: Make GC configurable?
        rt.set_gc_threshold(usize::MAX);
        // TODO: Add a comment here?
        let context = Context::full(&rt)?;
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
