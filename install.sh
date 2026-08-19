#!/usr/bin/env sh
# Direct installer for the `euglena` CLI from GitHub Releases.
#
#   curl -sSf https://raw.githubusercontent.com/codelovesme/euglena-cli/main/install.sh | sh
#
# For the broader codelovesme CLI namespace, prefer `cdlvsm install euglena`
# (https://github.com/codelovesme/cdlvsm-cli) — this script is the direct path,
# kept because it's simple and some setups just want exactly this.
#
# euglena runs apps through the `code` interpreter — install that too
# (`cdlvsm install code`, or https://github.com/codelovesme/code) and point
# euglena at it with `euglena code set <path>`.
#
# Env vars:
#   EUGLENA_VERSION  pin to a specific release tag (e.g. v0.1.0) instead of latest
#   PREFIX           install root (default: $HOME/.local); binary goes in $PREFIX/bin
set -eu

REPO="codelovesme/euglena-cli"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: '$1' is required but not found on PATH." >&2
        exit 1
    fi
}
need curl
need tar

os="$(uname -s)"
arch="$(uname -m)"
if [ "$os" != "Linux" ] || [ "$arch" != "x86_64" ]; then
    echo "error: prebuilt binaries are only available for Linux x86_64 (detected: $os $arch)." >&2
    echo "Build from source instead — see: https://github.com/$REPO#building-from-source" >&2
    exit 1
fi

if [ -n "${EUGLENA_VERSION:-}" ]; then
    tag="$EUGLENA_VERSION"
else
    echo "Fetching latest release info..."
    tag=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    if [ -z "$tag" ]; then
        echo "error: could not determine the latest release version." >&2
        echo "If a release hasn't been published yet, build from source instead." >&2
        exit 1
    fi
fi

asset="euglena-${tag}-x86_64-linux.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url..."
curl -fsSL "$url" -o "$tmp/$asset"

tar -xzf "$tmp/$asset" -C "$tmp"
stage_dir=$(find "$tmp" -maxdepth 1 -type d -name 'euglena-*')
if [ -z "$stage_dir" ]; then
    echo "error: unexpected archive layout — no euglena-* directory found." >&2
    exit 1
fi

mkdir -p "$BIN_DIR"
cp "$stage_dir/euglena" "$BIN_DIR/"
chmod +x "$BIN_DIR/euglena"

echo ""
echo "Installed to $BIN_DIR:"
"$BIN_DIR/euglena" --version

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        echo ""
        echo "Note: $BIN_DIR is not on your PATH. Add this to your shell profile:"
        echo "  export PATH=\"$BIN_DIR:\$PATH\""
        ;;
esac
