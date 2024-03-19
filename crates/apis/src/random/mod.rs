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
        });

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
