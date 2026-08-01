#!/usr/bin/env bash
set -e

# thing for nixshell and direnv
command -v direnv &> /dev/null && command -v nix &> /dev/null && direnv allow && eval "$(direnv export bash)"

cd "$(dirname "$0")"
DEST_DIR="../addons/lago/bin"
mkdir -p "$DEST_DIR"

export GDRUST_MAIN_EXTENSION="Lago"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

BUILD_LINUX=true
BUILD_WIN=false
PROFILE="debug"
CARGO_ARGS=""

# Parse args
for arg in "$@"; do
    case "$arg" in
        --all) BUILD_LINUX=true; BUILD_WIN=true ;;
        --win) BUILD_LINUX=false; BUILD_WIN=true ;;
        --release) PROFILE="release"; CARGO_ARGS="--release" ;;
        *) echo "Unknown flag: $arg"; exit 1 ;;
    esac
done

build_target() {
    local target=$1 filename=$2 os_name=$3
    local base="${filename%.*}"
    local ext="${filename##*.}" # Extensão do arquivo
    local suffix=$([[ "$PROFILE" == "debug" ]] && echo "_debug")

    echo "[ $os_name ${PROFILE^^} ]"
    if [[ "$os_name" == "WINDOWS" ]]; then
        cargo xwin build $CARGO_ARGS --target "$target"
    else
        cargo build $CARGO_ARGS --target "$target"
    fi

    install "$TARGET_DIR/$target/$PROFILE/$filename" "$DEST_DIR/${base}${suffix}.${ext}"
}

if $BUILD_LINUX; then build_target "x86_64-unknown-linux-gnu" "liblago.so" "LINUX"; fi
if $BUILD_WIN; then build_target "x86_64-pc-windows-msvc" "lago.dll" "WINDOWS"; fi
