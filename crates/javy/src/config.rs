use std::{
    fmt::Debug,
    io::{self, Write},
};

use anyhow::{bail, Result};
use bitflags::bitflags;

bitflags! {
    /// Flags to represent available JavaScript features.
    #[derive(Debug)]
    pub(crate) struct JSIntrinsics: u32  {
        const DATE = 1;
        const EVAL = 1 << 1;
        const REGEXP_COMPILER = 1 << 2;
        const REGEXP = 1 << 3;
        const JSON = 1 << 4;
        const PROXY = 1 << 5;
        const MAP_SET = 1 << 6;
        const TYPED_ARRAY  = 1 << 7;
        const PROMISE  = 1 << 8;
        const BIG_INT = 1 << 9;
        // Removed 10 and 11 representing BIG_FLOAT and BIG_DECIMAL.
        const OPERATORS = 1 << 12;
        const BIGNUM_EXTENSION = 1 << 13;
        const TEXT_ENCODING = 1 << 14;
        // Removed 15 representing STRING_NORMALIZE.
        const WEAK_REF = 1 << 16;
        const PERFORMANCE = 1 << 17;
    }
}

bitflags! {
    /// Flags representing implementation of JavaScript intrinsics
    /// made available through the `Javy` global.
    /// The APIs in this list can be thought of as APIs similar to the ones
    /// exposed by Node or Deno.
    ///
    /// NB: These APIs are meant to be migrated to a runtime-agnostic namespace,
    /// once efforts like WinterCG can be adopted.
    ///
    /// In the near future, Javy will include an extension mechanism, allowing
    /// users to extend the runtime with non-standard functionality directly
    /// from the CLI, at this point many, if not most, of these APIs will be
    /// moved out.
    #[derive(Debug)]
    pub(crate) struct JavyIntrinsics: u32 {
        const STREAM_IO = 1;
    }
}

/// A configuration for [`Runtime`](crate::Runtime).
///
/// These are the global configuration options to create a [`Runtime`](crate::Runtime),
/// and customize its behavior.
pub struct Config {
    /// JavaScript features.
    pub(crate) intrinsics: JSIntrinsics,
    /// Intrinsics exposed through the `Javy` namespace.
    pub(crate) javy_intrinsics: JavyIntrinsics,
    /// Whether to override the implementation of JSON.parse and JSON.stringify
    /// with a Rust implementation that uses a combination for Serde transcoding
    /// serde_json and simd_json.
    /// This setting requires the `JSON` intrinsic to be enabled, and the `json`
    /// crate feature to be enabled as well.
    pub(crate) simd_json_builtins: bool,
    /// The threshold to trigger garbage collection. Default is usize::MAX.
    pub(crate) gc_threshold: usize,
    /// The limit on the max amount of memory the runtime will use. Default is
    /// unlimited.
    pub(crate) memory_limit: usize,
    /// The limit on the max size of stack the runtime will use. Default is
    /// 256 * 1024.
    pub(crate) max_stack_size: usize,
    /// The stream to use for calls to `console.log`.
    pub(crate) log_stream: Box<dyn Write>,
    /// The stream to use for calls to `console.error`.
    pub(crate) err_stream: Box<dyn Write>,
}

impl Default for Config {
    /// Creates a [`Config`] with default values.
    fn default() -> Self {
        let mut intrinsics = JSIntrinsics::all();
        intrinsics.set(JSIntrinsics::TEXT_ENCODING, false);
        intrinsics.set(JSIntrinsics::WEAK_REF, false);
        intrinsics.set(JSIntrinsics::PERFORMANCE, false);
        Self {
            intrinsics,
            javy_intrinsics: JavyIntrinsics::empty(),
            simd_json_builtins: false,
            gc_threshold: usize::MAX,
            memory_limit: usize::MAX,
            max_stack_size: 256 * 1024, // from rquickjs
            log_stream: Box::new(std::io::stdout()),
            err_stream: Box::new(std::io::stderr()),
        }
    }
}

impl Config {
    /// Configures whether the JavaScript `Date` intrinsic will be available.
    pub fn date(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::DATE, enable);
        self
    }

    /// Configures whether the `Eval` intrinsic will be available.
    pub fn eval(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::EVAL, enable);
        self
    }

    /// Configures whether the regular expression compiler will be available.
    pub fn regexp_compiler(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::REGEXP_COMPILER, enable);
        self
    }

    /// Configures whether the `RegExp` intrinsic will be available.
    pub fn regexp(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::REGEXP, enable);
        self
    }

    /// Configures whether the QuickJS native JSON intrinsic will be
    /// available.
    pub fn json(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::JSON, enable);
        self
    }

    /// Configures whether proxy object creation  will be available.
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy
    pub fn proxy(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::PROXY, enable);
        self
    }

    /// Configures whether the `MapSet` intrinsic will be available.
    pub fn map_set(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::MAP_SET, enable);
        self
    }

    /// Configures whether the `Promise` instrinsic will be available.
    pub fn promise(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::PROMISE, enable);
        self
    }

    /// Configures whether supoort for `BigInt` will be available.
    pub fn big_int(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::BIG_INT, enable);
        self
    }

    /// Configures whether operator overloading wil be supported.
    pub fn operator_overloading(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::OPERATORS, enable);
        self
    }

    /// Configures whether extensions to `BigNum` will be available.
    pub fn bignum_extension(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::BIGNUM_EXTENSION, enable);
        self
    }

    /// Configures whether the `TextEncoding` and `TextDecoding` intrinsics will
    /// be available. NB: This is partial implementation.
    pub fn text_encoding(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::TEXT_ENCODING, enable);
        self
    }

    /// Whether the `Javy.IO` intrinsic will be available.
    /// Disabled by default. Note that it is strongly recommended to target
    /// WASI preview 1 when enabling this configuration. To use this
    /// configuration with WASI preview 2 or later, you cannot use Javy's
    /// plugin initialization and it will not be compatible for use with the
    /// Javy CLI.
    pub fn javy_stream_io(&mut self, enable: bool) -> &mut Self {
        self.javy_intrinsics.set(JavyIntrinsics::STREAM_IO, enable);
        self
    }

    /// Enables whether the output of console.log will be redirected to
    /// `stderr`.
    pub fn redirect_stdout_to_stderr(&mut self, enable: bool) -> &mut Self {
        self.log_stream = if enable {
            Box::new(io::stderr())
        } else {
            Box::new(io::stdout())
        };
        self
    }

    /// Whether to override the implementation of JSON.parse and JSON.stringify
    /// with a Rust implementation that uses a combination of Serde transcoding
    /// serde_json and simd_json for improved performance.
    /// This setting requires the `JSON` intrinsic to be enabled and the `json`
    /// crate feature to be enabled as well.
    /// Disabled by default.
    #[cfg(feature = "json")]
    pub fn simd_json_builtins(&mut self, enable: bool) -> &mut Self {
        self.simd_json_builtins = enable;
        self
    }

    /// The number of bytes to use to trigger garbage collection.
    /// The default is usize::MAX.
    pub fn gc_threshold(&mut self, bytes: usize) -> &mut Self {
        self.gc_threshold = bytes;
        self
    }

    /// The limit on the max amount of memory the runtime will use. Default is
    /// unlimited.
    pub fn memory_limit(&mut self, bytes: usize) -> &mut Self {
        self.memory_limit = bytes;
        self
    }

    /// The limit on the max size of stack the runtime will use. Default is
    /// 256 * 1024.
    pub fn max_stack_size(&mut self, bytes: usize) -> &mut Self {
        self.max_stack_size = bytes;
        self
    }

    /// The stream to use for calls to `console.log`.
    pub fn log_stream(&mut self, stream: Box<dyn Write>) -> &mut Self {
        self.log_stream = stream;
        self
    }

    /// The stream to use for calls to `console.error`.
    pub fn err_stream(&mut self, stream: Box<dyn Write>) -> &mut Self {
        self.err_stream = stream;
        self
    }

    /// Whether the `WeakRef` instrinsic will be enabled.
    pub fn weak_ref(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::WEAK_REF, enable);
        self
    }

    /// Whether the `Performance` intrinsic will be enabled.
    pub fn performance(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::PERFORMANCE, enable);
        self
    }

    pub(crate) fn validate(self) -> Result<Self> {
        if self.simd_json_builtins && !self.intrinsics.contains(JSIntrinsics::JSON) {
            bail!("JSON Intrinsic is required to override JSON.parse and JSON.stringify");
        }

        Ok(self)
    }
}

#[cfg(test)]
#[cfg(feature = "json")]
mod tests {
    use super::Config;

    #[test]
    fn err_config_validation() {
        let mut config = Config::default();
        config.simd_json_builtins(true);
        config.json(false);

        assert!(config.validate().is_err());
    }

    #[test]
    fn ok_config_validation() {
        let mut config = Config::default();
        config.simd_json_builtins(true);

        assert!(config.validate().is_ok());
    }
}
