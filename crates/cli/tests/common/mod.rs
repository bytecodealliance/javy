use anyhow::Result;
use javy_runner::Builder;

pub fn run_with_compile_and_build<F>(test: F) -> Result<()>
where
    F: Fn(&mut Builder) -> Result<()>,
{
    test(Builder::default().use_compile())?;
    test(&mut Builder::default())?;
    Ok(())
}
