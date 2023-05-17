use anyhow::Result;
use javy::{quickjs::JSValue, Runtime};

use crate::{APIConfig, JSApiSet};

pub struct Random;

impl JSApiSet for Random {
    fn register(&self, runtime: &Runtime, _config: &APIConfig) -> Result<()> {
        let ctx = runtime.context();
        ctx.global_object()?.get_property("Math")?.set_property(
            "random",
            // TODO figure out if we can lazily initialize the PRNG
            ctx.wrap_callback(|_ctx, _this, _args| Ok(JSValue::Float(fastrand::f64())))?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{random::Random, APIConfig, JSApiSet};
    use anyhow::Result;
    use javy::Runtime;

    #[test]
    fn test_random() -> Result<()> {
        let runtime = Runtime::default();
        Random.register(&runtime, &APIConfig::default())?;
        let ctx = runtime.context();
        ctx.eval_global("test.js", "result = Math.random()")?;
        let result = ctx.global_object()?.get_property("result")?.as_f64()?;
        assert!(result >= 0.0);
        assert!(result < 1.0);
        Ok(())
    }
}
