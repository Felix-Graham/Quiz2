#!/usr/bin/env bash
# install.sh - installer for French Vocab Quiz on Linux and ChromeOS (Crostini)
#
# What this does:
#   1. Makes sure Rust (cargo) is available, installing it via rustup if not.
#   2. Builds the quiz in release mode.
#   3. Copies the binary + vocab/ + a fresh config.toml into an install
#      folder (default: ~/.local/share/frenchquiz).
#   4. Symlinks the binary onto your PATH (~/.local/bin/frenchquiz) so you
#      can just type `frenchquiz` from anywhere.
#
# Usage:
#   ./install.sh                 # install to ~/.local/share/frenchquiz
#   FRENCHQUIZ_DIR=/opt/fq ./install.sh   # install elsewhere

set -euo pipefail

INSTALL_DIR="${FRENCHQUIZ_DIR:-$HOME/.local/share/frenchquiz}"
BIN_DIR="$HOME/.local/bin"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

say()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
warn() { printf '\033[1;33m!!\033[0m %s\n' "$1"; }
die()  { printf '\033[1;31mxx\033[0m %s\n' "$1"; exit 1; }

# ---------------------------------------------------------------------------
# 1. Make sure a C toolchain + curl exist (needed to build & to fetch rustup).
#    ChromeOS's Linux (Crostini) container is Debian-based, so apt covers it.
# ---------------------------------------------------------------------------
ensure_build_tools() {
    if command -v cc >/dev/null 2>&1 && command -v curl >/dev/null 2>&1; then
        return
    fi
    say "Installing build essentials (this needs sudo)..."
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update -y
        sudo apt-get install -y build-essential curl pkg-config git
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y gcc curl pkgconf-pkg-config git
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -Sy --noconfirm base-devel curl git
    else
        warn "Couldn't detect a package manager. Please make sure a C compiler, curl, and git are installed, then re-run."
    fi
}

# ---------------------------------------------------------------------------
# 2. Make sure cargo/rustc exist, installing via rustup if not.
# ---------------------------------------------------------------------------
ensure_rust() {
    if command -v cargo >/dev/null 2>&1; then
        say "Found Rust: $(rustc --version)"
        return
    fi
    say "Rust not found - installing via rustup (official, non-interactive)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
}

# ---------------------------------------------------------------------------
# 3. Build.
# ---------------------------------------------------------------------------
build() {
    say "Building French Vocab Quiz (release mode)..."
    (cd "$SCRIPT_DIR" && cargo build --release)
}

# ---------------------------------------------------------------------------
# 4. Install: copy binary + vocab + config template into INSTALL_DIR, then
#    symlink onto PATH.
# ---------------------------------------------------------------------------
install_files() {
    say "Installing into $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    cp "$SCRIPT_DIR/target/release/frenchquiz" "$INSTALL_DIR/frenchquiz"
    chmod +x "$INSTALL_DIR/frenchquiz"

    mkdir -p "$INSTALL_DIR/vocab"
    if [ -d "$SCRIPT_DIR/vocab" ]; then
        cp -n "$SCRIPT_DIR"/vocab/*.txt "$INSTALL_DIR/vocab/" 2>/dev/null || true
    fi

    mkdir -p "$BIN_DIR"
    ln -sf "$INSTALL_DIR/frenchquiz" "$BIN_DIR/frenchquiz"

    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            warn "$BIN_DIR isn't on your PATH yet."
            SHELL_RC="$HOME/.bashrc"
            [ -n "${ZSH_VERSION:-}" ] && SHELL_RC="$HOME/.zshrc"
            printf '\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$SHELL_RC"
            say "Added $BIN_DIR to PATH in $SHELL_RC. Restart your terminal or run: source $SHELL_RC"
            ;;
    esac
}

main() {
    say "French Vocab Quiz installer"
    ensure_build_tools
    ensure_rust
    build
    install_files
    say "Done! Run it with: frenchquiz"
    say "(if the command isn't found yet, restart your terminal first)"
}

main "$@"
