#!/usr/bin/env bash
set -euo pipefail

TARGET_TRIPLE="${1:-${TAURI_ENV_TARGET_TRIPLE:-${CARGO_BUILD_TARGET:-}}}"

if [[ -z "$TARGET_TRIPLE" ]]; then
  TARGET_TRIPLE="$(rustc -vV | awk '/host:/ {print $2}')"
fi

mkdir -p src-tauri/bin

(
  cd get-cookies-script
  bun build --compile src/index.ts --outfile ../src-tauri/bin/get-cookies
)

cp src-tauri/bin/get-cookies "src-tauri/bin/get-cookies-${TARGET_TRIPLE}"

echo "Built sidecar binaries:"
echo "  - src-tauri/bin/get-cookies"
echo "  - src-tauri/bin/get-cookies-${TARGET_TRIPLE}"
