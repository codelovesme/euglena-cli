//! `euglena`'s printed hints ("next steps", "set the code path") must name a
//! command that actually works for how the user invoked it. Under cdlvsm's
//! default install there is no bare `euglena` — only a `cdlvsm-euglena`
//! shim — so cdlvsm's dispatcher sets `CDLVSM_INVOKED_AS="cdlvsm euglena"`
//! and euglena must echo that back, not a hardcoded `euglena`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_euglena")
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("euglena_hint_{}_{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn init_hint_uses_bare_euglena_by_default() {
    let work = tmp_dir("bare");
    let out = Command::new(bin())
        .args(["init", "demo"])
        .current_dir(&work)
        .env_remove("CDLVSM_INVOKED_AS")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("euglena run"),
        "should hint bare `euglena run` with no CDLVSM_INVOKED_AS; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("cdlvsm euglena run"),
        "should not hint the dispatched form when not dispatched; got:\n{stdout}"
    );
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn init_hint_uses_cdlvsm_form_when_invoked_via_cdlvsm() {
    let work = tmp_dir("dispatched");
    let out = Command::new(bin())
        .args(["init", "demo"])
        .current_dir(&work)
        .env("CDLVSM_INVOKED_AS", "cdlvsm euglena")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cdlvsm euglena run"),
        "should hint `cdlvsm euglena run` when dispatched; got:\n{stdout}"
    );
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn code_show_hint_uses_cdlvsm_form_when_dispatched() {
    let home = tmp_dir("code_show_home");
    let out = Command::new(bin())
        .args(["code", "show"])
        .env("HOME", &home) // isolate: no real config
        .env("CDLVSM_INVOKED_AS", "cdlvsm euglena")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cdlvsm euglena code set"),
        "should hint the dispatched form; got:\n{stdout}"
    );
    let _ = fs::remove_dir_all(&home);
}
