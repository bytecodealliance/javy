// use crate::quickjs::JSContextRef;
use super::from_js_error;
use crate::{
    apis::{Console, NonStandardConsole, Random, StreamIO, TextEncoding},
    config::{JSIntrinsics, JavyIntrinsics},
    Config,
};

#[cfg(feature = "json")]
use crate::apis::{JavyJson, Json};

use anyhow::{bail, Result};
use rquickjs::{
    context::{intrinsic, Intrinsic},
    Context, Module, Runtime as QRuntime,
};
use std::mem::ManuallyDrop;

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

        // See comment above about configuring GC behaviour.
        rt.set_gc_threshold(usize::MAX);
        let context = Self::build_from_config(&rt, config)?;
        Ok(Self { inner: rt, context })
    }

    fn build_from_config(rt: &QRuntime, cfg: Config) -> Result<ManuallyDrop<Context>> {
        let cfg = cfg.validate()?;
        let intrinsics = &cfg.intrinsics;
        let javy_intrinsics = &cfg.javy_intrinsics;
        // We always set Random given that the principles around snapshotting and
        // random are applicable when using Javy from the CLI (the usage of
        // Wizer from the CLI is not optional).
        // NB: Users of Javy as a crate are welcome to switch this config,
        // however note that the usage of a custom `Random` implementation
        // should not affect the output of `Math.random()`.
        let context = Context::custom::<Random>(rt)?;

        // We use `Context::with` to ensure that there's a proper lock on the
        // context, making it totally safe to add the intrinsics below.
        context.with(|ctx| {
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

            if cfg.override_json_parse_and_stringify {
                #[cfg(feature = "json")]
                unsafe {
                    Json::add_intrinsic(ctx.as_raw())
                }
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
                unsafe { TextEncoding::add_intrinsic(ctx.as_raw()) }
            }

            if cfg.redirect_stdout_to_stderr {
                unsafe { NonStandardConsole::add_intrinsic(ctx.as_raw()) }
            } else {
                unsafe { Console::add_intrinsic(ctx.as_raw()) }
            }

            if javy_intrinsics.contains(JavyIntrinsics::STREAM_IO) {
                unsafe { StreamIO::add_intrinsic(ctx.as_raw()) }
            }

            if javy_intrinsics.contains(JavyIntrinsics::JSON) {
                #[cfg(feature = "json")]
                unsafe {
                    JavyJson::add_intrinsic(ctx.as_raw())
                }
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
