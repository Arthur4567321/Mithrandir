#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <dirname>" >&2
  exit 2
fi

TARGET="$1"
INSTALLED="/usr/local/bin/src/installed.json"

command -v jq >/dev/null 2>&1 || { echo "error: jq required" >&2; exit 1; }

# remove directory if it exists
if [ -e "$TARGET" ]; then
  rm -rf -- "$TARGET" || { echo "error: failed to remove $TARGET" >&2; exit 1; }
else
  echo "warning: $TARGET not found; continuing" >&2
fi

# ensure installed.json exists and is a valid structure
if [ ! -f "$INSTALLED" ]; then
  mkdir -p "$(dirname "$INSTALLED")"
  printf '{"packages": []}\n' > "$INSTALLED"
fi

# remove package entries whose dirname equals TARGET (treat missing dirname as "")
tmp="$(mktemp)"
jq --arg dir "$TARGET" '(.packages |= map(select((.dirname // "") != $dir)))' "$INSTALLED" > "$tmp" && mv "$tmp" "$INSTALLED"

echo "removed entry for dirname='$TARGET' from $INSTALLED"

