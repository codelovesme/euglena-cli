//! Tests for how `euglena` locates the `code` interpreter when no path has
//! been configured via `euglena code set` — it discovers cdlvsm's
//! `cdlvsm-code` shim on PATH, and deliberately never a bare `code` (which on
//! Linux is VS Code's CLI).

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_euglena")
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("euglena_disc_{}_{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write an executable shell script.
fn write_exec(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

/// A fake binary that reports a given `--version` line and echoes `marker` for
/// any other invocation (so a test can tell whether euglena actually ran it).
fn fake_bin(dir: &Path, name: &str, version_line: &str, marker: &str) {
    write_exec(
        &dir.join(name),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo '{version_line}'; exit 0; fi\necho {marker}\nexit 0\n"
        ),
    );
}

/// A minimal runnable project (no manifest.json, so `euglena run` invokes the
/// interpreter directly on the entry file).
fn minimal_project(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.code"), "x = 1\n").unwrap();
}

fn run_euglena_run(project: &Path, home: &Path, path_dirs: &Path) -> (bool, String) {
    let out = Command::new(bin())
        .args(["run", "src/main.code"])
        .current_dir(project)
        .env("HOME", home) // isolate: no real `euglena code set` config
        .env("PATH", path_dirs) // only our fake bin dir
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

#[test]
fn prefers_cdlvsm_code_over_a_vscode_impostor_on_path() {
    let work = tmp_dir("prefers_cdlvsm");
    let home = work.join("home");
    fs::create_dir_all(&home).unwrap();

    let fakebin = work.join("bin");
    fs::create_dir_all(&fakebin).unwrap();
    // The real interpreter shim (identifies as `Code v…`).
    fake_bin(
        &fakebin,
        "cdlvsm-code",
        "Code v9.9.9-fake",
        "CDLVSM_CODE_RAN",
    );
    // A decoy sharing the `code` name, VS-Code-style version.
    fake_bin(&fakebin, "code", "1.99.0", "VSCODE_RAN");

    let proj = work.join("proj");
    minimal_project(&proj);

    let (ok, out) = run_euglena_run(&proj, &home, &fakebin);
    assert!(
        ok,
        "euglena run should succeed via cdlvsm-code; got:\n{out}"
    );
    assert!(
        out.contains("CDLVSM_CODE_RAN"),
        "should invoke cdlvsm-code; got:\n{out}"
    );
    assert!(
        !out.contains("VSCODE_RAN"),
        "must NOT invoke VS Code's code; got:\n{out}"
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn refuses_a_lone_vscode_impostor_and_errors() {
    let work = tmp_dir("refuses_vscode");
    let home = work.join("home");
    fs::create_dir_all(&home).unwrap();

    let fakebin = work.join("bin");
    fs::create_dir_all(&fakebin).unwrap();
    // Only a VS-Code-like `code` on PATH — no real interpreter, no config.
    fake_bin(&fakebin, "code", "1.99.0", "VSCODE_RAN");

    let proj = work.join("proj");
    minimal_project(&proj);

    let (ok, out) = run_euglena_run(&proj, &home, &fakebin);
    assert!(!ok, "should fail with no real interpreter; got:\n{out}");
    assert!(
        out.contains("no Code interpreter"),
        "should print the no-interpreter error; got:\n{out}"
    );
    assert!(
        !out.contains("VSCODE_RAN"),
        "must never invoke VS Code's code; got:\n{out}"
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn does_not_auto_discover_a_bare_code_even_if_it_is_the_real_interpreter() {
    // Even a bare `code` that IS genuinely the interpreter is NOT auto-picked:
    // discovery is restricted to `cdlvsm-code` so euglena never has to reason
    // about whether a PATH `code` is the interpreter or VS Code. A non-cdlvsm
    // `code` must be pointed at explicitly with `euglena code set`.
    let work = tmp_dir("bare_code");
    let home = work.join("home");
    fs::create_dir_all(&home).unwrap();

    let fakebin = work.join("bin");
    fs::create_dir_all(&fakebin).unwrap();
    fake_bin(&fakebin, "code", "Code v0.4.1", "REAL_CODE_RAN");

    let proj = work.join("proj");
    minimal_project(&proj);

    let (ok, out) = run_euglena_run(&proj, &home, &fakebin);
    assert!(
        !ok,
        "bare `code` should not be auto-discovered; got:\n{out}"
    );
    assert!(
        out.contains("no Code interpreter"),
        "should print the no-interpreter error; got:\n{out}"
    );
    assert!(
        !out.contains("REAL_CODE_RAN"),
        "must not auto-run a bare `code`; got:\n{out}"
    );

    let _ = fs::remove_dir_all(&work);
}
