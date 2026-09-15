#!/usr/bin/env bash
set -euo pipefail

FUZZ_DIR="$(cd "$(dirname "$0")" && pwd)"
C_DIR="$(cd "$FUZZ_DIR/.." && pwd)"
IMAGE_NAME="uds-fuzz-stateless-ecu-mvp"

if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker is required." >&2
    exit 1
fi

docker build -t "$IMAGE_NAME" -f "$FUZZ_DIR/Dockerfile" "$FUZZ_DIR"

docker run --rm \
    --user "$(id -u):$(id -g)" \
    -v "$C_DIR:/work" \
    "$IMAGE_NAME"
