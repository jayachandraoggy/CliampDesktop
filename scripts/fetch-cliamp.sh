#!/usr/bin/env bash
set -euo pipefail

REPO="bjarneo/cliamp"
ASSET="cliamp-linux-amd64"
OUT="${1:-packaging/cliamp}"

mkdir -p "$(dirname "$OUT")"

URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
CHECKSUM_URL="https://github.com/${REPO}/releases/latest/download/checksums.txt"

TMP=$(mktemp)
CHECKSUMS=$(mktemp)
trap 'rm -f "$TMP" "$CHECKSUMS"' EXIT

if command -v curl >/dev/null; then
  curl -fSL -o "$TMP" "$URL"
  curl -fSL -o "$CHECKSUMS" "$CHECKSUM_URL"
elif command -v wget >/dev/null; then
  wget -qO "$TMP" "$URL"
  wget -qO "$CHECKSUMS" "$CHECKSUM_URL"
else
  echo "Error: curl or wget required" >&2
  exit 1
fi

EXPECTED=$(awk -v file="$ASSET" '$2 == file || $2 == "*" file { print $1 }' "$CHECKSUMS")
if [ -z "$EXPECTED" ]; then
  echo "Error: release has no checksum for ${ASSET}" >&2
  exit 1
fi

ACTUAL=$(sha256sum "$TMP" | awk '{print $1}')
if [ "$ACTUAL" != "$EXPECTED" ]; then
  echo "Error: checksum mismatch for ${ASSET}" >&2
  echo "  expected: $EXPECTED" >&2
  echo "  got:      $ACTUAL" >&2
  exit 1
fi

mv "$TMP" "$OUT"
chmod +x "$OUT"
trap - EXIT
echo "Fetched and verified ${ASSET} -> ${OUT}"
