#!/usr/bin/env bash
set -euo pipefail

PARTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$PARTS_DIR/../.." && pwd)"
ASSEMBLED_OUTPUT="${1:-$ROOT_DIR/doc/ZFSSYSTEM.md}"

require_contains() {
  local file="$1"
  local needle="$2"
  local label="$3"
  if ! grep -Fq -- "$needle" "$file"; then
    echo "snapshot validation failed: $label missing in $file" >&2
    echo "expected: $needle" >&2
    exit 1
  fi
}

require_absent() {
  local file="$1"
  local needle="$2"
  local label="$3"
  if grep -Fq -- "$needle" "$file"; then
    echo "snapshot validation failed: $label still present in $file" >&2
    echo "unexpected: $needle" >&2
    exit 1
  fi
}

require_path_absent() {
  local path="$1"
  local label="$2"
  if test -e "$path"; then
    echo "snapshot validation failed: $label still exists at $path" >&2
    exit 1
  fi
}

# Canonical identity: the index must declare the designation-bound output.
require_contains "$PARTS_DIR/_index.md" "**Designation:** ZFS" "index designation"
require_contains "$PARTS_DIR/_index.md" "BDS Documentation Protocol v2.0" "index protocol"
require_contains "$PARTS_DIR/_index.md" 'Primary output: `doc/ZFSSYSTEM.md`' "index primary output"
require_contains "$PARTS_DIR/BUILD.sh" 'DESIGNATION="ZFS"' "build designation"
require_absent  "$PARTS_DIR/_index.md" 'Primary output: `doc/SYSTEM.md`' "index legacy primary output"
require_absent  "$PARTS_DIR/_index.md" 'Command: `bash doc/SYSTEM.md`' "index legacy doc/SYSTEM.md command"
require_path_absent "$ROOT_DIR/SYSTEM.md" "legacy root snapshot"
require_path_absent "$ROOT_DIR/doc/SYSTEM.md" "legacy generic doc snapshot"
require_path_absent "$ROOT_DIR/doc/zsSYSTEM.md" "legacy designation snapshot"

# Assembled artifact must carry doctrine and not still declare legacy output.
test -f "$ASSEMBLED_OUTPUT"
require_contains "$ASSEMBLED_OUTPUT" "Document version" "assembled document version header"
require_contains "$ASSEMBLED_OUTPUT" "**Designation:** ZFS" "assembled designation"
require_contains "$ASSEMBLED_OUTPUT" 'Primary output: `doc/ZFSSYSTEM.md`' "assembled primary output"
require_contains "$ASSEMBLED_OUTPUT" "BDS Documentation Protocol v2.0" "assembled protocol"
require_contains "$ASSEMBLED_OUTPUT" "truth classes" "assembled truth classes"
require_absent  "$ASSEMBLED_OUTPUT" 'Primary output: `doc/SYSTEM.md`' "assembled legacy primary output"
require_absent  "$ASSEMBLED_OUTPUT" 'Root `SYSTEM.md` is the primary assembled reference.' "assembled legacy primary reference"

echo "snapshot validation passed: $ASSEMBLED_OUTPUT"
