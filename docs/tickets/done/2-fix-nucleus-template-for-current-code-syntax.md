# 2 — Fix the `euglena init` nucleus template field syntax

- **Priority:** High
- **Type:** Bug
- **Area:** `src/init.rs` (`nucleus_gene_template`), `tests/scaffold.rs`

Status: Fixed and shipped (2026-08-19).

## Problem

`euglena init <name>` scaffolds `src/nucleus.gene.code` from a hardcoded
template that declared the lifecycle type's field with a **colon**:

```
type EuglenaHasBeenBorn { cell_name:String }
```

`code` v0.3.0 rejects this with a parse error (`Unexpected: type
EuglenaHasBeenBorn { cell_name:Stri…`), so a freshly scaffolded app failed
immediately on `euglena run`. The scaffold's headline flow (`init` → `run`)
was broken out of the box.

## Root cause (two earlier wrong guesses corrected)

The bug is **only** the field-declaration operator: `code`'s `type` field
syntax uses `∈`, not a colon. Confirmed against `code`'s own
`tests/euglena/src/particles/lifecycle.code` **at the v0.3.0 tag**, which
declares `type EuglenaHasBeenBorn { cell_name ∈ String }`, and by running
both forms through the released `code` binary directly (colon → parse error,
`∈` → runs).

Earlier notes (in ticket 1 and this ticket's first draft) guessed the cause
was the `code` language having removed the `type` keyword (its T30), needing
a `Particle ∩ {...}` rewrite. **That was wrong** — v0.3.0 predates the entire
T26–T30 set-based rework and still uses `type`; it also has no `∩`/`Particle`
(both verified to parse-error / be undefined against the released binary). So
the minimal, correct fix keeps `type` and just switches the field operator.

## Fix

`src/init.rs`: `cell_name:String` → `cell_name ∈ String`. Nothing else in the
template changed — the handler, `return`, and `{ cell_name = cell_name }`
construction already used syntax `code` v0.3.0 accepts.

Verified end-to-end: `euglena code set <code>` → `euglena init demo` →
`euglena run` exits 0 (`Program executed successfully.`) against the real
released `code` v0.3.0.

## Regression guard

`tests/scaffold.rs`:
- `scaffolded_nucleus_uses_member_operator_not_colon` — hermetic, runs in
  CI: scaffolds via the built binary and asserts the nucleus gene uses
  `cell_name ∈ String` and not the colon form.
- `scaffolded_app_runs_against_real_code` — gated on `EUGLENA_TEST_CODE_BIN`
  (skipped in CI, which has no `code`): scaffolds and actually runs the app
  through a real `code` interpreter in an isolated `HOME`.
