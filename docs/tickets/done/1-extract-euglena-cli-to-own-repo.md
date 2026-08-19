# 1 — Extract the `euglena` CLI to its own public repo

Status: Implemented and shipped (initial v0.1.0, 2026-08-19).

## Why

`cdlvsm install euglena` (the package manager,
[`codelovesme/cdlvsm-cli`](https://github.com/codelovesme/cdlvsm-cli))
downloads release assets anonymously over `curl` from public GitHub
Releases. The `euglena` CLI previously lived only inside the
`codelovesme/euglena-platform` monorepo, which is **private** — so an
anonymous install could never fetch it, and the platform repo has no release
infrastructure at all (no release workflow, no releases, no product
version).

Rather than publish the whole private platform (runtime, 24 organelles,
`.env`s, etc.), the self-contained `euglena-cli` crate was extracted here —
the same pattern `code` and `cdlvsm` followed out of the same monorepo. The
platform keeps everything else private.

## What moved

The `euglena-cli` crate, unchanged in behavior: six source files
(`main.rs`, `exec.rs`, `config.rs`, `init.rs`, `codegen.rs`, `manifest.rs`),
depending only on `clap` + `serde_json`. Binary name: `euglena`.

- **No LLVM, no `code`-crate dependency.** The euglena binary invokes the
  `code` *interpreter binary* at runtime (path configured via `euglena code
  set <path>`, stored in `~/.config/euglena-cli/code_binary_path`) — it does
  not link the language. The platform monorepo's CI needs LLVM only because
  its *organelles* depend on the `code` crate; `euglena-cli` itself never
  did, which is what makes this a clean, lightweight extraction.
- **Left behind:** `assets/console.so` (a 4MB prebuilt organelle) — confirmed
  unused by any source path (the `console.so` references in `codegen.rs` are
  runtime app-relative manifest strings, not that asset file), so it wasn't
  worth carrying a stale binary into a fresh repo.
- Two trivial clippy lints (collapsible `if`, redundant closure) fixed so CI
  can enforce `clippy -D warnings`; these were never enforced in the
  monorepo. 7 unit tests carried over, all passing.

## Distribution

- `install.sh` — direct `curl | sh` bootstrap (`EUGLENA_VERSION` pins a tag).
- `.github/workflows/ci.yml` — fmt + clippy + build + test on push/PR. No
  LLVM or other system deps.
- `.github/workflows/release.yml` — tag-push `v*` (plus `workflow_dispatch`
  dry-run) builds `--release`, sanity-checks the binary, and uploads
  `euglena-<tag>-x86_64-linux.tar.gz` as a GitHub Release asset.
- `cdlvsm` points `Package::Euglena` at this repo's releases, so `cdlvsm
  install euglena` now works.

## Known follow-ups (not blocking)

- **`euglena init`'s nucleus template uses the `type` keyword**, which the
  `code` language removed in its T30. **Verified against the live release:
  `code` v0.3.0's interpreter rejects the template's
  `type EuglenaHasBeenBorn { cell_name:String }` line outright** (parse
  error at `type`), so a freshly `euglena init`'d app does NOT currently
  run via `euglena run` — it needs the template rewritten to current Code
  syntax (`EuglenaHasBeenBorn = Particle ∩ { cell_name ∈ String }`). This
  is a real known limitation, not a future-only concern; left unfixed here
  to keep the extraction a faithful move (the template came over verbatim),
  tracked as the first real follow-up for this repo. (An earlier note here
  claimed v0.3.0 predated the removal and would accept the template — that
  was wrong; the shipped v0.3.0 binary rejects it.)
- **euglena has no auto-discovery of `code` on `PATH`** — it strictly
  requires `euglena code set <path>`. A future convenience would be to fall
  back to `cdlvsm-code`/`code` on `PATH` when unconfigured, so `cdlvsm
  install euglena && cdlvsm install code` needs no manual wiring. Out of
  scope for the extraction.
