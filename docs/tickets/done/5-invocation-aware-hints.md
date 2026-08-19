# 5 — Printed hints must say the command that actually works (`cdlvsm euglena …`)

- **Priority:** Medium
- **Type:** UX
- **Area:** `src/invocation.rs` (new), `src/init.rs`, `src/config.rs`,
  `src/exec.rs`, `tests/invocation_hint.rs`, `README.md`; coordinated with
  `codelovesme/cdlvsm-cli`

Status: Fixed and shipped (2026-08-19).

## Problem

Under the recommended install (`cdlvsm install euglena`, no `--link`), cdlvsm
only creates a `cdlvsm-euglena` shim — there is **no bare `euglena` command**.
But every hint euglena printed (`euglena init`'s "next steps", `euglena code
show`'s unconfigured message, `exec.rs`'s error paths) said plain `euglena
…`, which doesn't exist for the majority of users who installed via cdlvsm
without `--link`. Confirmed live: `euglena init app` prints "euglena run" as
the next step, but running bare `euglena` fails (`command not found`) unless
you dispatch through `cdlvsm euglena run` or used `--link`.

## Fix

**`codelovesme/cdlvsm-cli` v0.2.2**: `dispatch()` in `src/main.rs` now sets
`CDLVSM_INVOKED_AS="cdlvsm <pkg>"` on the process it `exec()`s, so a
dispatched tool knows how it was actually invoked.

**This repo, v0.1.5**: new `src/invocation.rs::command_prefix()` reads
`CDLVSM_INVOKED_AS` (falling back to plain `"euglena"` when unset — i.e. a
direct/non-cdlvsm install, or `--link`ed). Every printed hint that used to
hardcode `euglena …` now uses this: `init.rs`'s "next steps", `config.rs`'s
unconfigured-interpreter message, and both hint sites in `exec.rs`
(`code_binary_path` failure, no-interpreter-found error). README rewritten
to lead with `cdlvsm euglena …` (the recommended path's actual working
command), noting `--link`/direct-install users can drop the prefix.

## Verification

- `tests/invocation_hint.rs` (hermetic, CI):
  - `init_hint_uses_bare_euglena_by_default` — no `CDLVSM_INVOKED_AS` → hints
    plain `euglena run`.
  - `init_hint_uses_cdlvsm_form_when_invoked_via_cdlvsm` — env var set →
    hints `cdlvsm euglena run`.
  - `code_show_hint_uses_cdlvsm_form_when_dispatched` — same for the
    `code show` unconfigured message.
- `cdlvsm-cli`'s own `tests/cli.rs::dispatch_sets_invoked_as_env_var` asserts
  the env var is actually set on the dispatched child process.
- Real end-to-end (both released): `cdlvsm install code && cdlvsm install
  euglena` → `cdlvsm euglena init app` prints "cdlvsm euglena run" as the
  next step (not bare `euglena run`) → `cdlvsm euglena run` succeeds.
