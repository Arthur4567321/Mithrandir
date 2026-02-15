#!/usr/bin/env bash
set -euo pipefail

# Usage: binary.sh <source-url> <archive-filename> <orig-dirname-or-empty> <version> <name>
if [ "$#" -ne 5 ]; then
  echo "usage: $0 <source> <archive> <orig_dirname_or_empty> <version> <name>" >&2
  exit 2
fi

SOURCE_URL="$1"
ARCHIVE_NAME="$2"
ORIG_DIRNAME="$3"
VERSION="$4"
NAME="$5"

MTR_STORE="mtr/store"
PACKAGES_FILE="src/packages.json"
INSTALLED_FILE="src/installed.json"

for cmd in curl tar jq; do
  command -v "$cmd" >/dev/null 2>&1 || { echo "error: $cmd required" >&2; exit 1; }
done

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

ARCHIVE_PATH="$TMPDIR/$ARCHIVE_NAME"
echo "Downloading $SOURCE_URL -> $ARCHIVE_PATH"
curl -L --fail --silent --show-error -o "$ARCHIVE_PATH" "$SOURCE_URL"

DEST="$MTR_STORE/${NAME}-${VERSION}"
rm -rf "$DEST"
mkdir -p "$DEST"

echo "Extracting $ARCHIVE_PATH -> $DEST"
tar -xf "$ARCHIVE_PATH" -C "$DEST"

# Ensure installed.json exists
if [ ! -f "$INSTALLED_FILE" ]; then
  mkdir -p "$(dirname "$INSTALLED_FILE")"
  printf '{"packages":[]}\n' > "$INSTALLED_FILE"
fi

# get dependencies and recipe for this package from packages.json
deps_json="$(jq -c --arg name "$NAME" '.packages[] | select(.name==$name) | (.dependencies // [])' "$PACKAGES_FILE" 2>/dev/null || echo '[]')"
recipe_json="$(jq -c --arg name "$NAME" '.packages[] | select(.name==$name) | (.recipe // null)' "$PACKAGES_FILE" 2>/dev/null || echo 'null')"
# ensure deps_json and recipe_json are valid JSON
if ! echo "$deps_json" | jq -e . >/dev/null 2>&1; then deps_json='[]'; fi
if ! echo "$recipe_json" | jq -e . >/dev/null 2>&1; then recipe_json='null'; fi

# Update installed.json: remove any existing entry with same name, then append new entry including dirname and recipe
tmp="$(mktemp)"
jq --arg name "$NAME" \
   --arg version "$VERSION" \
   --arg source "$SOURCE_URL" \
   --arg archive "$ARCHIVE_NAME" \
   --arg dirname "$MTR_STORE/$NAME-$VERSION" \
   --argjson deps "$deps_json" \
   --argjson recipe "$recipe_json" \
   '(.packages |= (map(select(.name != $name)) + [ {name:$name,version:$version,source:$source,archive:$archive,dirname:$dirname,dependencies:$deps,recipe:$recipe} ]))' \
   "$INSTALLED_FILE" > "$tmp" && mv "$tmp" "$INSTALLED_FILE"

