use anyhow::{Result};
use javy_embedded::Runtime;

fn main() -> Result<()> {
    let runtime = Runtime::default()?;
    let val = runtime.eval("script.js", "1 + 3")?;
    assert_eq!(4, val.as_i32_unchecked());
    runtime.eval("script.js", "console.log('hello world')")?;
    Ok(())
}