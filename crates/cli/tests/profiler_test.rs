#![cfg(feature = "profiler")]

use anyhow::{Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    str,
};
use tempfile::TempDir;

const REPORT_MAGIC: &[u8; 4] = b"JPRF";
const REPORT_HEADER_LEN: usize = 9;
const REPORT_RECORD_LEN: usize = 16;

// QuickJS opcode numbers, from the order of `DEF`s in `quickjs-opcode.h`.
const OP_CALL_METHOD: u32 = 36;
const OP_GET_VAR: u32 = 56;
const OP_GET_FIELD2: u32 = 65;

fn javy(args: &[&str], cwd: &Path) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_javy"))
        .current_dir(cwd)
        .args(args)
        .output()?)
}

/// Writes `js` to `dir` and builds it with `javy build`, passing
/// `build_args`. Returns the path of the module.
fn build(dir: &Path, js: &str, build_args: &[&str]) -> Result<PathBuf> {
    fs::write(dir.join("index.js"), js)?;
    let mut args = vec!["build", "index.js", "-o", "index.wasm"];
    args.extend_from_slice(build_args);
    let output = javy(&args, dir)?;
    ensure!(
        output.status.success(),
        "javy build failed: {}",
        str::from_utf8(&output.stderr)?
    );
    Ok(dir.join("index.wasm"))
}

// Instrumenting and running is slow, so a single test covers everything
// observable about a successful run.
#[test]
fn test_profile() -> Result<()> {
    let dir = TempDir::new()?;
    build(
        dir.path(),
        r#"
        function sum(n) {
            let s = 0;
            for (let i = 0; i < n; i++) s += i;
            return s;
        }
        console.log(`sum: ${sum(100)}`);
        "#,
        &[],
    )?;

    let output = javy(
        &[
            "profile",
            "index.wasm",
            "-o",
            "profile.txt",
            "--dump-trace",
            "trace.bin",
        ],
        dir.path(),
    )?;
    let stderr = str::from_utf8(&output.stderr)?;
    assert!(output.status.success(), "javy profile failed: {stderr}");

    // The program runs with the terminal's stdio, and nothing else is
    // written to it.
    assert_eq!("sum: 4950\n", str::from_utf8(&output.stdout)?);
    assert_eq!("", stderr);

    let trace = fs::read(dir.path().join("trace.bin"))?;
    assert!(trace.starts_with(REPORT_MAGIC), "bad trace magic");
    let records = u32::from_le_bytes(trace[5..9].try_into()?) as usize;
    assert!(records > 0, "the trace has no records");
    assert_eq!(
        REPORT_HEADER_LEN + records * REPORT_RECORD_LEN,
        trace.len(),
        "trace length does not match its record count"
    );

    // Records are keyed by the QuickJS opcode itself. `console.log(...)`
    // must show up as the opcodes it compiles to; an off-by-one or
    // rebased key would miss them.
    let (records_bytes, _) = trace[REPORT_HEADER_LEN..].as_chunks::<REPORT_RECORD_LEN>();
    let opcodes: Vec<u32> = records_bytes
        .iter()
        .map(|r| u32::from_le_bytes(r[4..8].try_into().unwrap()))
        .collect();
    for (name, opcode) in [
        ("OP_call_method", OP_CALL_METHOD),
        ("OP_get_var", OP_GET_VAR),
        ("OP_get_field2", OP_GET_FIELD2),
    ] {
        assert!(
            opcodes.contains(&opcode),
            "{name} ({opcode}) missing from {opcodes:?}"
        );
    }

    // One line per record, plus the column headings.
    let profile = fs::read_to_string(dir.path().join("profile.txt"))?;
    assert_eq!(records + 1, profile.lines().count());
    Ok(())
}

#[test]
fn test_profile_rejects_dynamically_linked_module() -> Result<()> {
    let dir = TempDir::new()?;
    let output = javy(&["emit-plugin", "-o", "plugin.wasm"], dir.path())?;
    ensure!(
        output.status.success(),
        "javy emit-plugin failed: {}",
        str::from_utf8(&output.stderr)?
    );
    build(
        dir.path(),
        "console.log(42);",
        &["-C", "dynamic", "-C", "plugin=plugin.wasm"],
    )?;

    let output = javy(&["profile", "index.wasm"], dir.path())?;
    assert!(!output.status.success());
    let stderr = str::from_utf8(&output.stderr)?;
    assert!(
        stderr.contains("No interpreter dispatch function found"),
        "unexpected error: {stderr}"
    );
    assert!(!dir.path().join("profile.txt").exists());
    Ok(())
}

#[test]
fn test_profile_help() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_javy"))
        .args(["profile", "--help"])
        .output()?;
    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout)?;
    for flag in ["-o <OUTPUT>", "--invoke <FUNCTION>", "--dump-trace <PATH>"] {
        assert!(stdout.contains(flag), "{flag} missing from help: {stdout}");
    }
    Ok(())
}
