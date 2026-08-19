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

Or install directly from a release:

```bash
curl -sSf https://raw.githubusercontent.com/codelovesme/euglena-cli/main/install.sh | sh
```

Linux x86_64 only, for now.

## Usage

```
euglena init <name>          scaffold a new app in ./<name>
euglena run [file]           run the app (default entry: src/main.code)
euglena build [file] [--release]   compile the app to a native binary
euglena test                 run the app's tests/ suite
euglena code set <path>      point euglena at your `code` interpreter binary
euglena code show
euglena code clear
```

### Finding `code`

`euglena run`/`build`/`test` shell out to the `code` interpreter. euglena
finds it automatically: if you haven't pinned a path, it uses cdlvsm's
`cdlvsm-code` shim from your `PATH`. So the usual flow needs no extra setup:

```bash
cdlvsm install code
cdlvsm install euglena
euglena init app && cd app && euglena run    # just works
```

It deliberately looks only for `cdlvsm-code`, never a bare `code` — on Linux
`code` is VS Code's own CLI, and euglena won't risk invoking that.

If your `code` came from somewhere other than cdlvsm (a direct `install.sh`,
a dev build, `cdlvsm install code --link`), point euglena at it explicitly
(stored in `~/.config/euglena-cli/code_binary_path`):

```bash
euglena code set /path/to/your/code
euglena code show      # what's configured
euglena code clear     # go back to cdlvsm-code discovery
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
