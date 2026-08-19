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
2. Otherwise `discover_cdlvsm_code_on_path()` scans `PATH` for **`cdlvsm-code`
   and only that name** — never a bare `code`. On Linux `code` is VS Code's
   own CLI, and euglena must never risk invoking it (or even running its
   `--version`). A `code` binary installed some other way (direct
   `install.sh`, a dev build, `cdlvsm install code --link`) is supported via
   an explicit `euglena code set`.
3. `is_code_interpreter()` still guards the discovered `cdlvsm-code` by
   running `--version` and requiring a `Code v…` prefix — a check against a
   broken/dangling shim resolving to something that isn't the interpreter.

> **Design note (owner):** an earlier cut of this (shipped as v0.1.3) also
> fell back to a bare `code` when it passed the `--version` guard. Narrowed
> to `cdlvsm-code`-only in **v0.1.4** — even *running* `--version` on an
> unknown PATH `code` is a risk not worth taking when `cdlvsm-code` is
> unambiguous.

Supporting changes: `euglena code show` (unconfigured) now explains PATH
discovery instead of implying a set is mandatory; `euglena init`'s "next
steps" no longer lists `euglena code set`; README rewritten around the
zero-config flow with the pin-a-path override documented after.

## Verification

- `tests/code_discovery.rs` (hermetic, CI): with fake binaries on an isolated
  PATH + HOME —
  - `prefers_cdlvsm_code_over_a_vscode_impostor_on_path` — picks `cdlvsm-code`,
    never the co-present `code`.
  - `refuses_a_lone_vscode_impostor_and_errors` — a lone `code` on PATH is
    never looked at; euglena errors cleanly without invoking it.
  - `does_not_auto_discover_a_bare_code_even_if_it_is_the_real_interpreter` —
    even a bare `code` reporting `Code v…` is NOT auto-picked; it needs an
    explicit `euglena code set`.
- Real end-to-end: `cdlvsm install code` + the built euglena, fresh HOME (no
  config), `~/.local/bin` on PATH → `euglena init && euglena run` →
  `Program executed successfully.` with no `euglena code set`.
