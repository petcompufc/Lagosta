#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"
RUST_DIR="$(pwd)"
IMAGE_NAME="lago-builder"

if command -v git >/dev/null 2>&1; then
    echo "BUILDING IN DOCKER"
    if ! docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
        echo "Building Docker image '$IMAGE_NAME'..."
        docker build -t "$IMAGE_NAME" .
    fi

    docker run --rm \
        -e CARGO_TERM_PROGRESS_WHEN=always \
        -e CARGO_TERM_PROGRESS_WIDTH=$COLUMNS \
        -v lago-target:/target \
        -v lago-cargo:/.cargo \
        -v "$RUST_DIR":/project \
        -v "$RUST_DIR/../addons":/addons \
        "$IMAGE_NAME" \
        bash -c './build.sh "$@" && chown -R '$(id -u):$(id -g)' /addons' -- "$@"
else
    echo "BUILDING IN HOST"
    ./build.sh "$@"
fi
