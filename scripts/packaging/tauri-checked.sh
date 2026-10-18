#!/usr/bin/env bash
# Preserve Tauri arguments and signing environment; check final Debian archives.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CHECK_DEB=false
BUNDLES=all
PROFILE=release
TARGET="${CARGO_BUILD_TARGET:-}"
BUNDLE_DISABLED=false
for ((i=1; i<=$#; i++)); do
    arg="${!i}"
    case "$arg" in
        --no-bundle) BUNDLE_DISABLED=true ;;
        --debug) PROFILE=debug ;;
        --bundles|-b|--target|-t)
            j=$((i + 1))
            value="${!j:-}"
            if [[ "$arg" == --target || "$arg" == -t ]]; then TARGET="$value"; else BUNDLES="$value"; fi
            ;;
        --bundles=*) BUNDLES="${arg#--bundles=}" ;;
        --target=*) TARGET="${arg#--target=}" ;;
    esac
done
if [[ "${1:-}" == build && "$(uname -s)" == Linux && "$BUNDLE_DISABLED" == false ]]; then
    if [[ ",$BUNDLES," == *,deb,* || "$BUNDLES" == all ]]; then CHECK_DEB=true; fi
fi
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/src-tauri/target}"
if [[ "$TARGET_DIR" != /* ]]; then TARGET_DIR="$ROOT/src-tauri/$TARGET_DIR"; fi
BUNDLE_DIR="$TARGET_DIR${TARGET:+/$TARGET}/$PROFILE/bundle/deb"
if [[ "$CHECK_DEB" == true ]]; then
    python3 scripts/packaging/lint_debs.py --check-tool
    BUILD_MARKER=$(mktemp "${TMPDIR:-/tmp}/bds-deb-build-XXXXXXXX")
    trap 'rm -f "$BUILD_MARKER"' EXIT
fi
npm exec -- tauri "$@"
if [[ "$CHECK_DEB" == true ]]; then
    shopt -s nullglob
    packages=("$BUNDLE_DIR/"*.deb)
    (( ${#packages[@]} > 0 )) || { echo "No Debian archive produced: $BUNDLE_DIR" >&2; exit 1; }
    for package in "${packages[@]}"; do
        if [[ ! -f "$package" || -L "$package" || ! "$package" -nt "$BUILD_MARKER" ]]; then
            echo "Debian output was not refreshed by this build: $package" >&2
            exit 1
        fi
    done
    REPORT_DIR=$(mktemp -d "${TMPDIR:-/tmp}/bds-lintian-XXXXXXXX")
    python3 scripts/packaging/lint_debs.py "$BUNDLE_DIR" \
        --report "$REPORT_DIR/report.json"
    echo "Lintian report: $REPORT_DIR/report.json"
fi
