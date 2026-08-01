FROM rust:1.97-slim-bullseye
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-pc-windows-msvc
RUN cargo install --locked cargo-xwin
RUN cargo xwin cache xwin

WORKDIR /project

ENV CARGO_HOME=/.cargo
ENV CARGO_TARGET_DIR=/target
ENV CARGO_TERM_COLOR=always
