FROM rust:1.53

RUN echo "deb http://apt.llvm.org/buster/ llvm-toolchain-buster main" >> /etc/apt/sources.list.d/llvm.list && \
  echo "deb-src http://apt.llvm.org/buster/ llvm-toolchain-buster main" >> /etc/apt/sources.list.d/llvm.list && \
  wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - && \
  apt-get update && \
  apt-get install -y build-essential cmake clang libclang-dev curl && \
  rustup component add clippy rustfmt && \
  rustup target add wasm32-wasi && \
  cargo install wasmtime-cli && \
  cargo install cargo-wasi

COPY . /app
