FROM rust:1.53

RUN apt-get update && \
  apt-get install -y build-essential cmake clang curl && \
  rustup component add clippy rustfmt && \
  rustup target add wasm32-wasi && \
  cargo install wasmtime-cli && \
  cargo install cargo-wasi

COPY . /app
