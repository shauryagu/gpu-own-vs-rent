#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA_DIR="${DATA_DIR:-$REPO_ROOT/data}"
CHI_BIN="${CHI_BIN:-$REPO_ROOT/target/release/chi}"

if [[ ! -x "$CHI_BIN" ]]; then
  echo "chi binary not found at $CHI_BIN; build with: cargo build -p chi --release" >&2
  exit 1
fi

PLIST_SRC="$REPO_ROOT/scripts/ocpi-hourly.plist"
PLIST_DST="$HOME/Library/LaunchAgents/com.chi.ocpi-hourly.plist"

mkdir -p "$(dirname "$PLIST_DST")"
mkdir -p "$DATA_DIR"

sed -e "s|__CHI_BIN__|$CHI_BIN|g" \
    -e "s|__REPO_ROOT__|$REPO_ROOT|g" \
    -e "s|__DATA_DIR__|$DATA_DIR|g" \
    "$PLIST_SRC" > "$PLIST_DST"

launchctl unload "$PLIST_DST" 2>/dev/null || true
launchctl load "$PLIST_DST"
echo "loaded $PLIST_DST"
