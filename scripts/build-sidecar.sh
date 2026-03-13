#!/usr/bin/env bash
set -euo pipefail

TARGET_TRIPLE="${1:-${TAURI_ENV_TARGET_TRIPLE:-${CARGO_BUILD_TARGET:-}}}"

if [[ -z "$TARGET_TRIPLE" ]]; then
  TARGET_TRIPLE="$(rustc -vV | awk '/host:/ {print $2}')"
fi

bun_compile_target_for_triple() {
  case "$1" in
    aarch64-apple-darwin) echo "bun-darwin-arm64" ;;
    x86_64-apple-darwin) echo "bun-darwin-x64" ;;
    aarch64-unknown-linux-gnu) echo "bun-linux-arm64" ;;
    x86_64-unknown-linux-gnu) echo "bun-linux-x64" ;;
    x86_64-pc-windows-msvc) echo "bun-windows-x64" ;;
    *) return 1 ;;
  esac
}

mkdir -p src-tauri/bin

COMPILE_ARGS=()
if BUN_TARGET="$(bun_compile_target_for_triple "$TARGET_TRIPLE")"; then
  COMPILE_ARGS+=(--target "$BUN_TARGET")
  echo "Building sidecar for Rust target '$TARGET_TRIPLE' using Bun target '$BUN_TARGET'"
else
  echo "Warning: no Bun compile target mapping for '$TARGET_TRIPLE', fallback to Bun host target" >&2
fi

(
  cd get-cookies-script
  bun build --compile "${COMPILE_ARGS[@]}" src/index.ts --outfile ../src-tauri/bin/get-cookies
)

cp src-tauri/bin/get-cookies "src-tauri/bin/get-cookies-${TARGET_TRIPLE}"

echo "Built sidecar binaries:"
echo "  - src-tauri/bin/get-cookies"
echo "  - src-tauri/bin/get-cookies-${TARGET_TRIPLE}"
