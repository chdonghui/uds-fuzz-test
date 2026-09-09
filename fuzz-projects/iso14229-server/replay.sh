#!/usr/bin/env bash
set -euo pipefail

FUZZ_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$FUZZ_DIR/../.." && pwd)"
IMAGE_NAME="uds-fuzz-iso14229-server"

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <artifact-name>" >&2
    exit 1
fi

ARTIFACT_NAME="$(basename "$1")"
HOST_ARTIFACT="$FUZZ_DIR/artifacts/$ARTIFACT_NAME"
CONTAINER_ARTIFACT="/work/fuzz-projects/iso14229-server/artifacts/$ARTIFACT_NAME"

if [ ! -f "$HOST_ARTIFACT" ]; then
    echo "ERROR: artifact not found: $HOST_ARTIFACT" >&2
    exit 1
fi

docker build -t "$IMAGE_NAME" -f "$FUZZ_DIR/Dockerfile" "$FUZZ_DIR"

docker run --rm \
    --user "$(id -u):$(id -g)" \
    -v "$PROJECT_ROOT:/work" \
    "$IMAGE_NAME" --replay "$CONTAINER_ARTIFACT"
