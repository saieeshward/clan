#!/usr/bin/env bash
# Set the workspace version — the single source of truth for the `clan` CLI
# version and (via tauri.conf.json's omitted version → Cargo fallback) every
# desktop bundle. Called from the release workflow with the pushed tag.
#
# Usage: set-version.sh <version>   e.g. set-version.sh 1.1.4
#
# Portable across the Linux/macOS/Windows GitHub runners: uses only bash + awk
# (Git Bash on Windows ships awk), so it needs no extra tooling to install.
set -euo pipefail

version="${1:?usage: set-version.sh <version>}"
version="${version#v}"

case "$version" in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) echo "error: '$version' is not a semver version" >&2; exit 1 ;;
esac

root="$(cd "$(dirname "$0")/../.." && pwd)"
cargo_toml="$root/Cargo.toml"

# Replace the first `version = "..."` line inside the [workspace.package] table
# only, leaving every other section untouched.
awk -v ver="$version" '
  /^\[/ { in_pkg = ($0 == "[workspace.package]") }
  in_pkg && !done && /^[[:space:]]*version[[:space:]]*=/ {
    sub(/"[^"]*"/, "\"" ver "\""); done = 1
  }
  { print }
  END { if (!done) { print "error: version line not found in [workspace.package]" > "/dev/stderr"; exit 1 } }
' "$cargo_toml" > "$cargo_toml.tmp"
mv "$cargo_toml.tmp" "$cargo_toml"

echo "set workspace version to $version"
