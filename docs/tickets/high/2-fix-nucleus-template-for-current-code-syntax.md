# 2 — Fix the `euglena init` nucleus template for current Code syntax

- **Priority:** High
- **Type:** Bug
- **Area:** `src/init.rs` (`nucleus_gene_template`)

Status: Open.

## Problem

`euglena init <name>` scaffolds `src/nucleus.gene.code` from a hardcoded
template (`src/init.rs::nucleus_gene_template`) that uses the Code language's
old `type` declaration syntax:

```
type EuglenaHasBeenBorn { cell_name:String }

booted = false

EuglenaHasBeenBorn => {
    booted = true
    return EuglenaHasBeenBorn { cell_name = cell_name }
}
```

The Code language **removed the `type` keyword** (code repo's T30 — particle
types are now plain `∩`-merged Schema variables on a predefined `Particle`
base). Verified against the live release: `code` **v0.3.0 rejects the `type`
line with a parse error**, so a freshly scaffolded app fails immediately on
`euglena run`:

```
error: Unexpected: type EuglenaHasBeenBorn { cell_name:String }
```

So `euglena init` currently produces a non-running app. This makes the
scaffold's headline flow (`init` → `run`) broken out of the box.

## Fix

Rewrite the template to current Code syntax. The direct translation of the
particle type is:

```
EuglenaHasBeenBorn = Particle ∩ { cell_name ∈ String }

booted = false

EuglenaHasBeenBorn => {
    booted = true
    return EuglenaHasBeenBorn { cell_name = cell_name }
}
```

(Confirm the handler/`return`/construction syntax against a current `code`
release too — only the `type` declaration was checked here. Add a smoke
test that scaffolds a project and runs it against an installed `code` so
this can't silently rot again.)

## Notes

- This was inherited verbatim from the `euglena-platform` monorepo during
  the extraction (see ticket 1) and left unchanged there to keep the
  extraction faithful — the template staleness predates this repo.
- Coordinate with the Code language's actual current syntax at fix time
  rather than trusting this ticket's snippet; the language is evolving.
