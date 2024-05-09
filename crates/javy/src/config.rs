use bitflags::bitflags;

bitflags! {
    /// Flags to represent available JavaScript features.
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
        const BIG_FLOAT = 1 << 10;
        const BIG_DECIMAL = 1 << 11;
        const OPERATORS = 1 << 12;
        const BIGNUM_EXTENSION = 1 << 13;
        const TEXT_ENCODING = 1 << 14;
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
    /// Whether to use a custom console implementation provided by Javy,
    /// that redirects stdout to stderr.
    pub(crate) redirect_stdout_to_stderr: bool,
}

impl Default for Config {
    /// Creates a [`Config`] with default values.
    fn default() -> Self {
        let mut intrinsics = JSIntrinsics::all();
        intrinsics.set(JSIntrinsics::TEXT_ENCODING, false);
        Self {
            intrinsics,
            javy_intrinsics: JavyIntrinsics::empty(),
            redirect_stdout_to_stderr: false,
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

    /// Configures whether support for `BigFloat` will be available.
    pub fn big_float(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::BIG_FLOAT, enable);
        self
    }

    /// Configures whether supporr for `BigDecimal` will be available.
    pub fn big_decimal(&mut self, enable: bool) -> &mut Self {
        self.intrinsics.set(JSIntrinsics::BIG_DECIMAL, enable);
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
    /// Disabled by default.
    pub fn javy_stream_io(&mut self, enable: bool) -> &mut Self {
        self.javy_intrinsics.set(JavyIntrinsics::STREAM_IO, enable);
        self
    }

    /// Enables whether the output of console.log will be redirected to
    /// `stderr`.
    pub fn redirect_stdout_to_stderr(&mut self, enable: bool) -> &mut Self {
        self.redirect_stdout_to_stderr = enable;
        self
    }
}
