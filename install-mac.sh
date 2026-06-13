#!/usr/bin/env bash
set -euo pipefail

REPO="saieeshward/clan"
BIN_NAME="clan"
BIN_DIR="${CLAN_BIN_DIR:-/usr/local/bin}"

# Detect arch
ARCH="$(uname -m)"
case "$ARCH" in
  arm64)  TARGET="aarch64-apple-darwin" ;;
  x86_64) TARGET="x86_64-apple-darwin" ;;
  *)
    echo "error: unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Fetch latest version
echo "Fetching latest release..."
VERSION="${CLAN_VERSION:-}"
if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/p' | head -n1)"
  if [ -z "$VERSION" ]; then
    echo "error: could not determine latest version."
    exit 1
  fi
fi

TARBALL="clan-v${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"

echo "Installing clan v${VERSION} (${TARGET})..."

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL --progress-bar "$URL" -o "$TMP/$TARBALL"
tar -xzf "$TMP/$TARBALL" -C "$TMP"

BIN_SRC="$TMP/clan-v${VERSION}-${TARGET}/clan"

if [ -w "$BIN_DIR" ]; then
  install -m 755 "$BIN_SRC" "$BIN_DIR/$BIN_NAME"
else
  sudo install -m 755 "$BIN_SRC" "$BIN_DIR/$BIN_NAME"
fi

# Clear Gatekeeper quarantine
xattr -d com.apple.quarantine "$BIN_DIR/$BIN_NAME" 2>/dev/null || true

echo ""
echo "✓ clan v${VERSION} installed to $BIN_DIR/$BIN_NAME"
echo ""
echo "Try it: clan --help"
