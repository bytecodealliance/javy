use anyhow::{Error, Result};
use javy::{
    quickjs::{prelude::Func, Object},
    Runtime,
};

use crate::{APIConfig, JSApiSet};

pub struct Random;

impl JSApiSet for Random {
    fn register(&self, runtime: &Runtime, _config: &APIConfig) -> Result<()> {
        runtime.context().with(|cx| {
            let globals = cx.globals();
            let math: Object<'_> = globals.get("Math").expect("Math global to be defined");
            math.set("random", Func::from(fastrand::f64))?;

            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{random::Random, APIConfig, JSApiSet};
    use anyhow::{Error, Result};
    use javy::{
        quickjs::{context::EvalOptions, Value},
        Runtime,
    };

    #[test]
    fn test_random() -> Result<()> {
        let runtime = Runtime::default();
        Random.register(&runtime, &APIConfig::default())?;
        runtime.context().with(|this| {
            let eval_opts = EvalOptions {
                strict: false,
                ..Default::default()
            };
            this.eval_with_options("result = Math.random()", eval_opts)?;
            let result: f64 = this
                .globals()
                .get::<&str, Value<'_>>("result")?
                .as_float()
                .unwrap();
            assert!(result >= 0.0);
            assert!(result < 1.0);
            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}
