mod bytecode;
mod commands;
mod exports;
mod js;
mod wasm_generator;
mod wit;

use crate::commands::{Command, CompileCommandOpts, EmitProviderCommandOpts};
use crate::wasm_generator::r#static as static_generator;
use anyhow::{bail, Result};
use exports::Export;
use js::JS;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Stdio;
use std::{fs, process::Command as OsCommand};
use structopt::StructOpt;
use wasm_generator::dynamic as dynamic_generator;

fn main() -> Result<()> {
    let cmd = Command::from_args();

    match &cmd {
        Command::EmitProvider(opts) => emit_provider(opts),
        Command::Compile(opts) => {
            let js = JS::from_file(&opts.input)?;
            let exports = match (&opts.wit, &opts.wit_world) {
                (None, None) => Ok(vec![]),
                (None, Some(_)) => Ok(vec![]),
                (Some(_), None) => bail!("Must provide WIT world when providing WIT file"),
                (Some(wit), Some(world)) => exports::process_exports(&js, wit, world),
            }?;
            if opts.dynamic {
                let wasm = dynamic_generator::generate(&js, exports)?;
                fs::write(&opts.output, wasm)?;
            } else {
                create_statically_linked_module(opts, exports)?;
            }
            Ok(())
        }
    }
}

fn emit_provider(opts: &EmitProviderCommandOpts) -> Result<()> {
    let mut file: Box<dyn Write> = match opts.out.as_ref() {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };
    file.write_all(bytecode::QUICKJS_PROVIDER_MODULE)?;
    Ok(())
}

fn create_statically_linked_module(opts: &CompileCommandOpts, exports: Vec<Export>) -> Result<()> {
    // The javy-core `main.rs` pre-initializer uses WASI to read the JS source
    // code from stdin. Wizer doesn't let us customize its WASI context so we
    // don't have a better option right now. Since we can't set the content of
    // the stdin stream for the main Javy process, we create a subprocess and
    // set the content of the subprocess's stdin stream, and we run Wizer
    // within that subprocess. The subprocess runs the same Javy command but
    // with an env var set, in both cases we end up in this method, and we
    // can use the env var to tell if we're not in the subprocess and we need
    // to start the subprocess or we are in the subprocess and we can run Wizer.
    const IN_SUBPROCESS_KEY: &str = "JAVY_WIZEN";
    const IN_SUBPROCESS_VAL: &str = "1";
    let in_subprocess = env::var(IN_SUBPROCESS_KEY).is_ok_and(|v| v == IN_SUBPROCESS_VAL);
    let wasm = if !in_subprocess {
        let js = JS::from_file(&opts.input)?;

        let mut args = env::args();
        let self_cmd = args.next().unwrap();
        let mut command = OsCommand::new(self_cmd)
            .args(args)
            .env(IN_SUBPROCESS_KEY, IN_SUBPROCESS_VAL)
            .stdin(Stdio::piped())
            .spawn()?;
        command.stdin.take().unwrap().write_all(js.as_bytes())?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }

        // The subprocess should have written some Wasm so we can refine it now.
        let wizened_wasm = fs::read(&opts.output)?;
        static_generator::refine(wizened_wasm, &js, exports)?
    } else {
        static_generator::generate()?
    };

    fs::write(&opts.output, wasm)?;
    Ok(())
}
