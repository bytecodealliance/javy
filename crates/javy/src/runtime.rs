// use crate::quickjs::JSContextRef;
use super::from_js_error;
#[cfg(feature = "json")]
use crate::apis::json;
use crate::{
    apis::{console, random, stream_io, text_encoding},
    config::{JSIntrinsics, JavyIntrinsics},
    Config,
};

use anyhow::{bail, Result};
use rquickjs::{
    context::{intrinsic, Intrinsic},
    Context, Module, Runtime as QRuntime,
};
use std::{
    io::{stderr, stdout},
    mem::ManuallyDrop,
};

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
    pub fn new(config: Config) -> Result<Self> {
        let rt = ManuallyDrop::new(QRuntime::new()?);

        let context = Self::build_from_config(&rt, config)?;
        Ok(Self { inner: rt, context })
    }

    fn build_from_config(rt: &QRuntime, cfg: Config) -> Result<ManuallyDrop<Context>> {
        let cfg = cfg.validate()?;
        let intrinsics = &cfg.intrinsics;
        let javy_intrinsics = &cfg.javy_intrinsics;

        rt.set_gc_threshold(cfg.gc_threshold);
        rt.set_memory_limit(cfg.memory_limit);
        rt.set_max_stack_size(cfg.max_stack_size);

        // Using `Context::base` seems to have a bug where it tries to register
        // the same intrinsic twice.
        let context = Context::custom::<()>(rt)?;

        // We use `Context::with` to ensure that there's a proper lock on the
        // context, making it totally safe to add the intrinsics below.
        context.with(|ctx| {
            // We always set Random given that the principles around snapshotting and
            // random are applicable when using Javy from the CLI (the usage of
            // Wizer from the CLI is not optional).
            // NB: Users of Javy as a crate are welcome to switch this config,
            // however note that the usage of a custom `Random` implementation
            // should not affect the output of `Math.random()`.
            random::register(ctx.clone()).expect("registering `random` APIs to succeed");

            if intrinsics.contains(JSIntrinsics::DATE) {
                unsafe { intrinsic::Date::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::EVAL) {
                unsafe { intrinsic::Eval::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::REGEXP_COMPILER) {
                unsafe { intrinsic::RegExpCompiler::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::REGEXP) {
                unsafe { intrinsic::RegExp::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::JSON) {
                unsafe { intrinsic::Json::add_intrinsic(ctx.as_raw()) }
            }

            #[cfg(feature = "json")]
            if cfg.simd_json_builtins {
                json::register(ctx.clone()).expect("registering JSON builtins to succeed");
            }

            if intrinsics.contains(JSIntrinsics::PROXY) {
                unsafe { intrinsic::Proxy::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::MAP_SET) {
                unsafe { intrinsic::MapSet::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::TYPED_ARRAY) {
                unsafe { intrinsic::TypedArrays::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::PROMISE) {
                unsafe { intrinsic::Promise::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::BIG_INT) {
                unsafe { intrinsic::BigInt::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::BIG_FLOAT) {
                unsafe { intrinsic::BigFloat::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::BIG_DECIMAL) {
                unsafe { intrinsic::BigDecimal::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::BIGNUM_EXTENSION) {
                unsafe { intrinsic::BignumExt::add_intrinsic(ctx.as_raw()) }
            }

            if intrinsics.contains(JSIntrinsics::TEXT_ENCODING) {
                text_encoding::register(ctx.clone())
                    .expect("registering TextEncoding APIs to succeed");
            }

            if cfg.redirect_stdout_to_stderr {
                console::register(ctx.clone(), stderr(), stderr())
                    .expect("registering console to succeed");
            } else {
                console::register(ctx.clone(), stdout(), stderr())
                    .expect("registering console to succeed");
            }

            if javy_intrinsics.contains(JavyIntrinsics::STREAM_IO) {
                stream_io::register(ctx.clone())
                    .expect("registering StreamIO functions to succeed");
            }

            #[cfg(feature = "json")]
            if javy_intrinsics.contains(JavyIntrinsics::JSON) {
                json::register_javy_json(ctx.clone())
                    .expect("registering Javy.JSON builtins to succeed");
            }
        });

        Ok(ManuallyDrop::new(context))
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
            .with(|this| Module::declare(this.clone(), name, contents)?.write_le())
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
