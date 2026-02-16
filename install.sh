#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-/usr/local/bin}"
SRC_DIR="$PREFIX/src"
STORE_DIR="$PREFIX/mtr/store"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build mtr" >&2
  exit 1
fi

if [ "$(id -u)" -eq 0 ]; then
  SUDO=""
else
  if command -v sudo >/dev/null 2>&1; then
    SUDO="sudo"
  else
    echo "error: this installer needs permission to write to $PREFIX (run as root or install sudo)" >&2
    exit 1
  fi
fi

echo "Building mtr (release)..."
cargo build --release

echo "Installing binary to $PREFIX/mtr"
$SUDO install -Dm755 target/release/mtr "$PREFIX/mtr"

echo "Installing runtime files to $SRC_DIR"
$SUDO install -d "$SRC_DIR" "$STORE_DIR"
$SUDO install -m644 packages.json "$SRC_DIR/packages.json"
$SUDO install -m644 installed.json "$SRC_DIR/installed.json"
$SUDO install -m755 binary.sh "$SRC_DIR/binary.sh"
$SUDO install -m755 source.sh "$SRC_DIR/source.sh"
$SUDO install -m755 remove.sh "$SRC_DIR/remove.sh"

echo "Install complete. You can now run: mtr <package>"
