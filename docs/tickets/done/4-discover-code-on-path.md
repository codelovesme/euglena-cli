# 4 — Discover `code` on PATH instead of requiring `euglena code set`

- **Priority:** Medium
- **Type:** UX
- **Area:** `src/exec.rs`, `src/config.rs`, `src/init.rs`, `README.md`,
  `tests/code_discovery.rs`

Status: Fixed and shipped (2026-08-19).

## Problem

`euglena run`/`build`/`test` resolved the `code` interpreter *only* from the
path set via `euglena code set` (stored in
`~/.config/euglena-cli/code_binary_path`). With nothing configured it errored
out. So even the normal cdlvsm flow —

```
cdlvsm install code
cdlvsm install euglena
euglena init app && euglena run
```

— failed until the user manually ran `euglena code set …/cdlvsm-code`, an
avoidable step since both binaries already land on PATH (`~/.local/bin`).

## Fix

`src/exec.rs::find_code_binary_or_exit` now falls back to PATH discovery when
no path is configured:

1. Explicit configured path (from `euglena code set`) still wins — it's the
   override for dev builds / custom locations, no longer a requirement.
2. Otherwise `discover_code_on_path()` scans `PATH`, preferring `cdlvsm-code`
   (checked across all PATH dirs first) then a bare `code`.
3. Only a candidate that identifies itself as the interpreter is accepted:
   `is_code_interpreter()` runs `<candidate> --version` and requires the
   output to start with `Code v`. **This is the key safety guard** — it means
   a same-named impostor like VS Code's `code` CLI (whose `--version` prints
   a bare number like `1.99.0`) is never invoked. Name-preference dominating
   PATH order plus the version check together make the collision the whole
   cdlvsm `--link` design worries about a non-issue here.

Supporting changes: `euglena code show` (unconfigured) now explains PATH
discovery instead of implying a set is mandatory; `euglena init`'s "next
steps" no longer lists `euglena code set`; README rewritten around the
zero-config flow with the pin-a-path override documented after.

## Verification

- `tests/code_discovery.rs` (hermetic, CI): with fake binaries on an isolated
  PATH + HOME —
  - `prefers_cdlvsm_code_over_a_vscode_impostor_on_path` — picks `cdlvsm-code`,
    never the VS-Code-style `code`.
  - `refuses_a_lone_vscode_impostor_and_errors` — a lone VS-Code-like `code`
    is rejected (version guard) and euglena errors cleanly without invoking it.
  - `discovers_bare_code_when_it_is_the_real_interpreter` — a bare `code` that
    reports `Code v…` is accepted.
- Real end-to-end: `cdlvsm install code` + the built euglena, fresh HOME (no
  config), `~/.local/bin` on PATH → `euglena init && euglena run` →
  `Program executed successfully.` with no `euglena code set`.
