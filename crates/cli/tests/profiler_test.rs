#![cfg(feature = "profiler")]

use anyhow::{Result, ensure};
use javy_profiler::format;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    str,
};
use tempfile::TempDir;

// QuickJS opcode numbers, from the order of `DEF`s in `quickjs-opcode.h`.
const OP_GET_VAR: u32 = 56;
const OP_GET_FIELD2: u32 = 65;
const OP_INITIAL_YIELD: u32 = 134;
const OP_YIELD: u32 = 135;
// Encoded sizes, from the `DEF`s in `quickjs-opcode.h`.
const OP_GET_VAR_SIZE: u32 = 5;
// `initial_yield` and `yield` are both this size.
const OP_YIELD_SIZE: u32 = 1;

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
        function thrower() { throw new Error("boom"); }
        function middle() { thrower(); }
        let caught = false;
        try { middle(); } catch (e) { caught = true; }
        function* gen() { yield 1; yield 2; }
        let total = 0;
        for (const v of gen()) total += v;
        console.log(`sum: ${sum(100)}, caught: ${caught}, gen: ${total}`);
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
    assert_eq!(
        "sum: 4950, caught: true, gen: 3\n",
        str::from_utf8(&output.stdout)?
    );
    assert_eq!("", stderr);

    let report = format::read(&fs::read(dir.path().join("trace.bin"))?)?;
    assert!(!report.sites.is_empty(), "the trace has no sites");

    // One entry hint per distinct activation start: the top level, `sum`,
    // `middle`, `thrower`, and `gen` four times. Calling `gen()` runs it
    // from byte 0 to its implicit `initial_yield`; each of the three
    // `next()` calls then resumes it right after the previous
    // `initial_yield` or `yield`. Calls into C functions never dispatch,
    // so they record none.
    assert_eq!(
        8,
        report.entries.len(),
        "unexpected entries {:?}",
        report.entries
    );
    // Every entry is the address of a dispatch, so it is also a site.
    for entry in &report.entries {
        assert!(
            report.sites.iter().any(|s| s.addr == entry.addr),
            "entry {:#x} is not a site",
            entry.addr
        );
    }

    // Resumed activations enter mid-buffer, right after the suspending
    // opcode: once after `initial_yield` and twice after a `yield`.
    let resumes_after = |opcode| {
        report
            .sites
            .iter()
            .filter(|s| s.opcode == opcode)
            .filter(|s| {
                report
                    .entries
                    .iter()
                    .any(|e| e.addr == s.addr + OP_YIELD_SIZE)
            })
            .count()
    };
    assert_eq!(1, resumes_after(OP_INITIAL_YIELD));
    assert_eq!(2, resumes_after(OP_YIELD));

    // `console.log` compiles to `get_var console` immediately followed by
    // `get_field2 log`: the opcode and its address must both be right.
    let has_site = |addr, opcode| {
        report
            .sites
            .iter()
            .any(|s| s.addr == addr && s.opcode == opcode)
    };
    assert!(
        report
            .sites
            .iter()
            .any(|s| s.opcode == OP_GET_VAR && has_site(s.addr + OP_GET_VAR_SIZE, OP_GET_FIELD2)),
        "no `get_var` followed by `get_field2` in {:?}",
        report.sites
    );

    let profile = fs::read_to_string(dir.path().join("profile.txt"))?;
    assert_eq!(report.sites.len() + 1, profile.lines().count());
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
