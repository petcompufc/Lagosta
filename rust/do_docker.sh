#!/bin/bash
# Run a command inside the docker container.

cd "$(dirname "$0")"
RUST_DIR="$(pwd)"
IMAGE_NAME="lago-builder"

docker run --rm \
    -e CARGO_TERM_PROGRESS_WHEN=always \
    -e CARGO_TERM_PROGRESS_WIDTH=$COLUMNS \
    -v lago-target:/target \
    -v lago-cargo:/.cargo \
    -v "$RUST_DIR":/project \
    -v "$RUST_DIR/../addons":/addons \
    "$IMAGE_NAME" \
    bash -c "$*"
