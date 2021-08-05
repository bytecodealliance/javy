FROM rust:1.52

RUN apt-get update && \
  apt-get install -y build-essential cmake clang && \
  rustup component add clippy rustfmt && \
  rustup target add wasm32-wasi

COPY . /app
