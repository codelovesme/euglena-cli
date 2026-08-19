//! Integration tests for `euglena init` scaffolding.
//!
//! The hermetic tests run everywhere (CI included). The real-run test is gated
//! behind `EUGLENA_TEST_CODE_BIN=/path/to/code` — the only way to *truly*
//! verify the scaffolded template is valid Code is to run it through a real
//! `code` interpreter, which CI doesn't have.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_euglena")
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("euglena_it_{}_{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// `euglena init` scaffolds a project whose nucleus gene uses current Code
/// particle-schema syntax (`Particle ∩ { _class ∈ "…", … }`), not the removed
/// `type` keyword. Getting this wrong is what made `euglena init` →
/// `euglena run` fail (see T2, T3).
#[test]
fn scaffolded_nucleus_uses_current_particle_syntax() {
    let work = tmp_dir("nucleus_syntax");
    let status = Command::new(bin())
        .args(["init", "demo"])
        .current_dir(&work)
        .status()
        .unwrap();
    assert!(status.success(), "euglena init failed");

    let gene = fs::read_to_string(work.join("demo/src/nucleus.gene.code")).unwrap();

    // Must build the type from the `Particle` base via `∩`, pinning `_class`,
    // with the field typed via `∈`.
    assert!(
        gene.contains("Particle ∩ {") && gene.contains("_class ∈ \"EuglenaHasBeenBorn\""),
        "nucleus gene should use `Particle ∩ {{ _class ∈ … }}`; got:\n{gene}"
    );
    assert!(
        gene.contains("cell_name ∈ String"),
        "nucleus gene should declare the field with `∈`; got:\n{gene}"
    );
    // Must NOT use the removed `type` keyword or the colon field form.
    assert!(
        !gene.contains("type EuglenaHasBeenBorn") && !gene.contains("cell_name:String"),
        "nucleus gene must not use the removed `type` keyword or colon field; got:\n{gene}"
    );

    let _ = fs::remove_dir_all(&work);
}

/// Full end-to-end: scaffold a project and actually run it through a real
/// `code` interpreter. Gated on `EUGLENA_TEST_CODE_BIN` since CI has no `code`.
#[test]
fn scaffolded_app_runs_against_real_code() {
    let code_bin = match std::env::var("EUGLENA_TEST_CODE_BIN") {
        Ok(p) if !p.is_empty() => p,
        _ => {
            eprintln!("skipping: set EUGLENA_TEST_CODE_BIN=/path/to/code to run");
            return;
        }
    };

    let work = tmp_dir("real_run");

    // Point euglena at the code binary. `euglena code set` writes to
    // ~/.config/euglena-cli — isolate it via a temp HOME so the test doesn't
    // clobber the developer's real config.
    let home = work.join("home");
    fs::create_dir_all(&home).unwrap();

    let set = Command::new(bin())
        .args(["code", "set", &code_bin])
        .env("HOME", &home)
        .status()
        .unwrap();
    assert!(set.success(), "euglena code set failed");

    let init = Command::new(bin())
        .args(["init", "demo"])
        .current_dir(&work)
        .env("HOME", &home)
        .status()
        .unwrap();
    assert!(init.success(), "euglena init failed");

    let run = Command::new(bin())
        .arg("run")
        .current_dir(work.join("demo"))
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "euglena run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );

    let _ = fs::remove_dir_all(&work);
}
