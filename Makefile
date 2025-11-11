.PHONY: fmt fmt-check lint-wasi-targets test-wasi-targets wasi-targets lint-native-targets test-native-targets native-targets test-wpt test clean cli plugin build-test-plugins build-default-plugin vet ci
.DEFAULT_GOAL := cli

# === Format checks ===
fmt-check:
	cargo fmt --all --check
fmt:
	cargo fmt --all

# === Lint & Test WASI Targets ===
lint-wasi-targets: fmt-check
	cargo clippy --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-fuzz \
	--target=wasm32-wasip2 --all-targets --all-features -- -D warnings

test-wasi-targets:
	cargo hack test --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-fuzz \
	--exclude=javy-test-plugin-wasip2 \
	--exclude=javy-test-invalid-plugin \
	--target=wasm32-wasip2 --each-feature -- --nocapture

wasi-targets: lint-wasi-targets test-wasi-targets

# === Lint & Test Native Targets ===
lint-native-targets: fmt-check build-default-plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy --workspace \
	--exclude=javy \
	--exclude=javy-plugin-api \
	--exclude=javy-plugin \
	--exclude=javy-test-invalid-plugin \
	--exclude=javy-test-plugin-wasip2 \
	--release --all-targets --all-features -- -D warnings

test-native-targets: build-default-plugin build-test-plugins
	CARGO_PROFILE_RELEASE_LTO=off cargo hack test --workspace \
	--exclude=javy \
	--exclude=javy-plugin-api \
	--exclude=javy-plugin \
	--exclude=javy-test-invalid-plugin \
	--exclude=javy-test-plugin-wasip2 \
	--release --each-feature -- --nocapture

native-targets: lint-native-targets test-native-targets

# === Web Platform Tests

test-wpt: cli
	npm install --prefix wpt
	npm test --prefix wpt

# === All tests ===
test-all: wasi-targets native-targets test-wpt

# === Binaries ===

# First, build the default plugin, which is a dependency to the CLI.
# No need to run `javy_plugin_processing`, the CLI build.rs will take
# care of doing that.
target/release/javy: build-default-plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo build -p=javy-cli --release

target/wasm32-wasip2/release/plugin.wasm:
	cargo build -p=javy-plugin --target=wasm32-wasip2 --release

target/wasm32-wasip2/release/test_plugin.wasm:
	cargo build -p=javy-test-plugin-wasip2 --target=wasm32-wasip2 --release
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip2/release/test_plugin.wasm target/wasm32-wasip2/release/test_plugin.wasm

target/wasm32-unknown-unknown/release/test_invalid_plugin.wasm:
	cargo build -p=javy-test-invalid-plugin --target=wasm32-unknown-unknown --release

cli: target/release/javy

# Build the default plugin
build-default-plugin: target/wasm32-wasip2/release/plugin.wasm

# Build auxiliary plugins, for testing
build-test-plugins: target/wasm32-wasip2/release/plugin.wasm target/wasm32-wasip2/release/test_plugin.wasm target/wasm32-unknown-unknown/release/test_invalid_plugin.wasm

# === Misc ===
clean:
	cargo clean

vet:
	cargo vet --locked

# Intended to simulate what the GitHub Actions CI workflow will run.
# We don't invoke this directly because we often run out of disk space in
# GitHub Actions if we try to compile native targets in the same workflow as
# WASI targets so we have to use a multi-step process in GitHub to avoid that.
ci: lint-wasi-targets lint-native-targets vet test-all
