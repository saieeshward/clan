#!/usr/bin/env bash
set -euo pipefail

REPO="saieeshward/clan"
BIN_DIR="/usr/local/bin"
BIN_NAME="clan"

# Resolve the version to install. Override with CLAN_VERSION=1.2.3 to pin one;
# otherwise fetch the latest published release tag from GitHub so a one-line
# install always tracks the newest release instead of a hardcoded one.
VERSION="${CLAN_VERSION:-}"
if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/p' | head -n1)"
  if [ -z "$VERSION" ]; then
    echo "Could not determine the latest release; set CLAN_VERSION to install a specific version." >&2
    exit 1
  fi
fi

# Detect OS and arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "Windows users: download the .msi from https://github.com/$REPO/releases"
    exit 1
    ;;
esac

TARBALL="clan-v${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"

echo "Installing clan v${VERSION} for ${TARGET}..."

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/$TARBALL"
echo "File Downloaded"
tar -xzf "$TMP/$TARBALL" -C "$TMP"
echo $(ls $TMP/$TARBALL)

# Install binary
if [ -w "$BIN_DIR" ]; then
  install -m 755 "$TMP/clan-v${VERSION}-x86_64-unknown-linux-gnu" "$BIN_DIR/$BIN_NAME"
else
  sudo install -m 755 "$TMP/clan-v${VERSION}-x86_64-unknown-linux-gnu/clan" "$BIN_DIR/$BIN_NAME"
fi

# Clear macOS quarantine
if [ "$OS" = "Darwin" ]; then
  xattr -d com.apple.quarantine "$BIN_DIR/$BIN_NAME" 2>/dev/null || true
fi

echo "clan $(clan --version) installed to $BIN_DIR/$BIN_NAME"
