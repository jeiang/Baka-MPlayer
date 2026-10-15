#!/usr/bin/env bash
# PROTOTYPE (throwaway). Usage: ./run.sh [shot.png]   env: PROTO_RUST_SEEKBAR=1, PROTO_VIDEO=<file|url>
# Headless by default (Xvfb + Mesa llvmpipe). Unset PROTO_HEADLESS to use the real display.
set -euo pipefail
cd "$(dirname "$0")"
NIX="nix --extra-experimental-features nix-command --extra-experimental-features flakes develop path:. -c"
$NIX cargo build
if [ "${PROTO_HEADLESS-1}" = 1 ]; then
  PROTO_SHOT="${1:-$PWD/shots/run.png}" QT_QPA_PLATFORM=xcb $NIX xvfb-run -a -s "-screen 0 1280x720x24 +extension GLX" ./target/debug/proto-main-window
else
  $NIX ./target/debug/proto-main-window
fi
