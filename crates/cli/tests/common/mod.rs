use anyhow::Result;
use javy_runner::{Builder, JavyCommand};

pub fn run_with_compile_and_build<F>(test: F) -> Result<()>
where
    F: Fn(&mut Builder) -> Result<()>,
{
    test(Builder::default().command(JavyCommand::Compile))?;
    test(Builder::default().command(JavyCommand::Build))?;
    Ok(())
}
