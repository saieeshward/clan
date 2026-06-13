#!/usr/bin/env bash
set -euo pipefail

REPO="saieeshward/clan"
BIN_NAME="clan"
BIN_DIR="${CLAN_BIN_DIR:-/usr/local/bin}"

# Detect OS and arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "error: unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "error: unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  *)
    echo "error: unsupported OS: $OS"
    echo "Windows users: download the .msi from https://github.com/$REPO/releases"
    exit 1
    ;;
esac

# Resolve version — pin with CLAN_VERSION=1.2.3 or fetch latest
VERSION="${CLAN_VERSION:-}"
if [ -z "$VERSION" ]; then
  echo "Fetching latest release..."
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/p' | head -n1)"
  if [ -z "$VERSION" ]; then
    echo "error: could not determine latest version. Set CLAN_VERSION to install a specific version."
    exit 1
  fi
fi

# ── CLI install ────────────────────────────────────────────────────────────────

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

if [ "$OS" = "Darwin" ]; then
  xattr -d com.apple.quarantine "$BIN_DIR/$BIN_NAME" 2>/dev/null || true
fi

echo ""
echo "✓ clan v${VERSION} installed to $BIN_DIR/$BIN_NAME"
echo ""
cat << 'EOF'
   ██████╗██╗      █████╗ ███╗   ██╗
  ██╔════╝██║     ██╔══██╗████╗  ██║
  ██║     ██║     ███████║██╔██╗ ██║
  ██║     ██║     ██╔══██║██║╚██╗██║
  ╚██████╗███████╗██║  ██║██║ ╚████║
   ╚═════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝

  Context and Live Agent Notation
  Any model. Any framework. One file.
EOF

# ── Viewer (interactive) ───────────────────────────────────────────────────────

echo ""
printf "Would you like to install the CLAN Viewer (desktop app)? [y/N] "
read -r INSTALL_VIEWER </dev/tty

if [[ "$INSTALL_VIEWER" =~ ^[Yy]$ ]]; then
  DOWNLOADS="${HOME}/Downloads"
  mkdir -p "$DOWNLOADS"

  case "$OS" in
    Darwin)
      VIEWER_FILE="CLAN.Viewer_${VERSION}_universal.dmg"
      VIEWER_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${VIEWER_FILE}"
      VIEWER_DEST="$DOWNLOADS/$VIEWER_FILE"

      echo "Downloading CLAN Viewer..."
      curl -fsSL --progress-bar "$VIEWER_URL" -o "$VIEWER_DEST"

      echo ""
      echo "✓ CLAN Viewer downloaded to $VIEWER_DEST"
      echo ""
      echo "To install:"
      echo "  1. Open $VIEWER_DEST"
      echo "  2. Drag CLAN Viewer to your Applications folder"
      echo "  3. On first launch, clear the Gatekeeper warning:"
      echo "     xattr -d com.apple.quarantine \"/Applications/CLAN Viewer.app\""
      echo "     open \"/Applications/CLAN Viewer.app\""
      echo ""
      echo "To open any .clan file in the viewer:"
      echo "  open -a \"CLAN Viewer\" your-file.clan"

      # Offer to open the DMG immediately
      echo ""
      printf "Open the DMG now? [y/N] "
      read -r OPEN_DMG </dev/tty
      if [[ "$OPEN_DMG" =~ ^[Yy]$ ]]; then
        open "$VIEWER_DEST"
      fi
      ;;

    Linux)
      VIEWER_FILE="CLAN.Viewer_${VERSION}_amd64.AppImage"
      VIEWER_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${VIEWER_FILE}"
      VIEWER_DEST="$DOWNLOADS/$VIEWER_FILE"

      echo "Downloading CLAN Viewer..."
      curl -fsSL --progress-bar "$VIEWER_URL" -o "$VIEWER_DEST"
      chmod +x "$VIEWER_DEST"

      echo ""
      echo "✓ CLAN Viewer downloaded to $VIEWER_DEST"
      echo ""
      echo "To launch:"
      echo "  $VIEWER_DEST"
      echo ""
      echo "To make it easier to run, create an alias:"
      echo "  echo \"alias clan-viewer='$VIEWER_DEST'\" >> ~/.bashrc && source ~/.bashrc"
      echo ""
      echo "To open any .clan file:"
      echo "  $VIEWER_DEST your-file.clan"
      ;;
  esac
else
  echo ""
  echo "Skipping viewer. You can install it later from:"
  echo "  https://github.com/$REPO/releases"
fi

echo ""
echo "Get started: clan --help"
