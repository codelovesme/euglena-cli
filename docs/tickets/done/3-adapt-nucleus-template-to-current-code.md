# 3 — Adapt the nucleus template to current Code (post-`type` syntax)

- **Priority:** High
- **Type:** Bug / compatibility
- **Area:** `src/init.rs` (`nucleus_gene_template`), `tests/scaffold.rs`

Status: Fixed and shipped (2026-08-19).

## Context

T2 fixed the scaffold template to run against the **released** `code` v0.3.0
(colon → `∈` field operator, keeping the `type` keyword). But v0.3.0 is far
behind: `code`'s `main` was 38 commits ahead, including the whole T26–T30
set-based rework. In current Code there is **no `type` keyword at all** — a
particle type is a plain schema built from the predefined `Particle` base
with `∩`, pinning `_class` explicitly:

```
EuglenaHasBeenBorn = Particle ∩ { _class ∈ "EuglenaHasBeenBorn", cell_name ∈ String }
```

So the T2 template (`type EuglenaHasBeenBorn { cell_name ∈ String }`) runs on
v0.3.0 but is **rejected by current code**. euglena must target current Code,
not the stale release (owner's call: don't hold Code back to fit euglena —
adapt euglena forward). Coordinated with a fresh `code` **v0.4.0** release
cut from `main`, which `cdlvsm install code` now delivers.

## Fix

`src/init.rs`: nucleus template's type declaration rewritten from
`type EuglenaHasBeenBorn { cell_name ∈ String }` to
`EuglenaHasBeenBorn = Particle ∩ { _class ∈ "EuglenaHasBeenBorn", cell_name ∈ String }`.
The handler, `return`, and `{ cell_name = cell_name }` construction were
already current-syntax-valid and are unchanged.

Derived and verified empirically against a local build of current `code`
`main` (not guessed): `Particle` is predefined, `∩` merges schemas, and the
scaffolded gene + an `emit … to this get …` round-trips (`booted = true`,
`result.cell_name` preserved). Confirmed the whole `euglena init` →
`euglena run` flow exits 0 against current code.

## Regression guard

`tests/scaffold.rs::scaffolded_nucleus_uses_current_particle_syntax` now
asserts the gene uses `Particle ∩ { … }` with `_class ∈ "EuglenaHasBeenBorn"`
and `cell_name ∈ String`, and does **not** contain the removed `type`
keyword or a colon field. The gated `scaffolded_app_runs_against_real_code`
test (run with `EUGLENA_TEST_CODE_BIN` pointing at a current `code`)
confirms it actually runs.

## Relationship to T2

T2's colon→`∈` fix was correct for v0.3.0 but is superseded here: the target
moved from the stale release to current Code. Net template change from the
original monorepo form is: drop `type`, build from `Particle ∩`, pin
`_class`, keep `∈` for the field.
