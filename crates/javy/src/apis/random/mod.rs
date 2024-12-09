use crate::quickjs::{prelude::Func, Ctx, Object};
use anyhow::{Error, Result};

/// Register a `random` object on the global object that seeds itself at first
/// execution.
pub(crate) fn register(cx: Ctx) -> Result<()> {
    let globals = cx.globals();
    let math: Object<'_> = globals.get("Math").expect("Math global to be defined");
    math.set("random", Func::from(fastrand::f64))?;

    Ok::<_, Error>(())
}

#[cfg(test)]
mod tests {
    use crate::{
        quickjs::{context::EvalOptions, Value},
        Runtime,
    };
    use anyhow::{Error, Result};

    #[test]
    fn test_random() -> Result<()> {
        let runtime = Runtime::default();
        runtime.context().with(|this| {
            let mut eval_opts = EvalOptions::default();
            eval_opts.strict = false;
            this.eval_with_options::<(), _>("result = Math.random()", eval_opts)?;
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
