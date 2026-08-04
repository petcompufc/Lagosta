FROM rust:slim-bookworm

# Base build tools (removed lld – we'll get it from LLVM 19)
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc-mingw-w64-x86-64 \
    pkg-config \
    cmake \
    ninja-build \
    wget \
    lsb-release \
    software-properties-common \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install LLVM 19 (all packages) and set up symlinks for the tools cargo-xwin needs
RUN wget https://apt.llvm.org/llvm.sh \
    && chmod +x llvm.sh \
    && ./llvm.sh 19 all \
    && rm llvm.sh \
    # compiler
    && ln -sf /usr/bin/clang-19     /usr/bin/clang \
    && ln -sf /usr/bin/clang++-19   /usr/bin/clang++ \
    # linker
    && ln -sf /usr/bin/lld-19       /usr/bin/lld \
    && ln -sf /usr/bin/lld-link-19  /usr/bin/lld-link \
    # static library archiver & indexer
    && ln -sf /usr/bin/llvm-lib-19  /usr/bin/llvm-lib \
    && ln -sf /usr/bin/llvm-ar-19   /usr/bin/llvm-ar \
    && ln -sf /usr/bin/llvm-ranlib-19 /usr/bin/llvm-ranlib

# Add Rust targets
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-pc-windows-msvc

# Install cross‑compilation helper
RUN cargo install --locked cargo-xwin
RUN cargo xwin cache xwin

WORKDIR /project

ENV CARGO_HOME=/.cargo
ENV CARGO_TARGET_DIR=/target
ENV CARGO_TERM_COLOR=always
