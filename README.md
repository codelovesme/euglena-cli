# euglena

The `euglena` CLI — scaffold, run, and build [Euglena](https://github.com/codelovesme)
applications. Euglena apps are written in the [Code](https://github.com/codelovesme/code)
language; this CLI wraps the `code` toolchain with the Euglena app model
(a `manifest.json` cell definition plus `*.gene.code` genes and organelles).

## Install

The recommended way is via [`cdlvsm`](https://github.com/codelovesme/cdlvsm-cli),
the package manager for codelovesme CLI tools:

```bash
cdlvsm install euglena
cdlvsm install code      # euglena runs apps through the code interpreter
```

This gives you `cdlvsm euglena <command>` (dispatched through `cdlvsm`) — by
design there's **no bare `euglena` command** unless you pass `--link`:

```bash
cdlvsm install euglena --link   # also get a bare `euglena` on PATH
```

Or install directly from a release, which does put a bare `euglena` on PATH:

```bash
curl -sSf https://raw.githubusercontent.com/codelovesme/euglena-cli/main/install.sh | sh
```

Linux x86_64 only, for now. Examples below use `cdlvsm euglena` — drop the
`cdlvsm ` prefix if you installed with `--link` or `install.sh`.

## Usage

```
cdlvsm euglena init <name>          scaffold a new app in ./<name>
cdlvsm euglena run [file]           run the app (default entry: src/main.code)
cdlvsm euglena build [file] [--release]   compile the app to a native binary
cdlvsm euglena test                 run the app's tests/ suite
cdlvsm euglena code set <path>      point euglena at a specific `code` binary
cdlvsm euglena code show
cdlvsm euglena code clear
```

```bash
cdlvsm euglena init app
cd app
cdlvsm euglena run    # just works — no code-path setup needed, see below
```

### Finding `code`

`run`/`build`/`test` shell out to the `code` interpreter. euglena finds it
automatically: if you haven't pinned a path, it uses cdlvsm's `cdlvsm-code`
shim from your `PATH` — no extra setup needed after `cdlvsm install code`.

It deliberately looks only for `cdlvsm-code`, never a bare `code` — on Linux
`code` is VS Code's own CLI, and euglena won't risk invoking that.

If your `code` came from somewhere other than cdlvsm (a direct `install.sh`,
a dev build, `cdlvsm install code --link`), point euglena at it explicitly
(stored in `~/.config/euglena-cli/code_binary_path`):

```bash
cdlvsm euglena code set /path/to/your/code
cdlvsm euglena code show      # what's configured
cdlvsm euglena code clear     # go back to cdlvsm-code discovery
```

### App layout

`euglena init myapp` creates:

```
myapp/
  manifest.json          cell name + organelles
  src/nucleus.gene.code  boot gene (any src/*.gene.code is auto-linked)
```

Organelles (native `.so`/`.wasm` capability modules) are resolved from
`euglena-organelles/` directories in the project's ancestor folders and
declared in `manifest.json`.

## Building from source

```bash
cargo build --release    # ./target/release/euglena
cargo test
```

Depends only on `clap` and `serde_json` — no LLVM, no build-time dependency
on the `code` language (it invokes the `code` binary at runtime).

## License

GPL-3.0-or-later — see [LICENSE](./LICENSE).
