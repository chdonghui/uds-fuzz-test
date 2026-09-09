#!/usr/bin/env bash
set -euo pipefail

EXAMPLE_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$EXAMPLE_DIR/../.." && pwd)"
IMAGE_NAME="uds-fuzz-libfuzzer-basic"

docker build -t "$IMAGE_NAME" -f "$EXAMPLE_DIR/Dockerfile" "$EXAMPLE_DIR"

docker run --rm \
    --user "$(id -u):$(id -g)" \
    -v "$PROJECT_ROOT:/work" \
    "$IMAGE_NAME"
