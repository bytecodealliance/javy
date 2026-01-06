.PHONY: fmt fmt-check lint-wasi-targets lint-wasip1-targets lint-wasip2-targets \
	test-wasi-targets test-wasip1-targets test-wasip2-targets wasi-targets \
	lint-native-targets test-native-targets test-native-targets-ci native-targets \
	test-wpt test-wpt-ci test-all clean cli build-default-plugin build-test-plugins \
	vet ci
.DEFAULT_GOAL := cli

# === Format checks ===
fmt-check:
	cargo fmt --all --check
fmt:
	cargo fmt --all

# === Lint & Test WASI Targets ===
lint-wasi-targets: fmt-check lint-wasip1-targets lint-wasip2-targets

lint-wasip1-targets:
	cargo clippy --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-test-plugin-wasip2 \
	--exclude=javy-fuzz \
	--target=wasm32-wasip1 --all-targets --all-features -- -D warnings

lint-wasip2-targets:
	cargo clippy --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-test-plugin-wasip1 \
	--exclude=javy-fuzz \
	--target=wasm32-wasip2 --all-targets --all-features -- -D warnings

test-wasi-targets: test-wasip1-targets test-wasip2-targets

test-wasip1-targets:
	cargo hack test --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-fuzz \
	--exclude=javy-test-plugin-wasip1 \
	--exclude=javy-test-plugin-wasip2 \
	--exclude=javy-test-invalid-plugin \
	--target=wasm32-wasip1 --each-feature -- --nocapture

test-wasip2-targets:
	cargo hack test --workspace \
	--exclude=javy-cli \
	--exclude=javy-codegen \
	--exclude=javy-plugin \
	--exclude=javy-plugin-processing \
	--exclude=javy-runner \
	--exclude=javy-fuzz \
	--exclude=javy-test-plugin-wasip1 \
	--exclude=javy-test-plugin-wasip2 \
	--exclude=javy-test-invalid-plugin \
	--target=wasm32-wasip2 --each-feature -- --nocapture

wasi-targets: lint-wasi-targets test-wasi-targets

# === Lint & Test Native Targets ===
lint-native-targets: build-default-plugin lint-native-targets-ci

lint-native-targets-ci: fmt-check
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy --workspace \
	--exclude=javy \
	--exclude=javy-plugin-api \
	--exclude=javy-plugin \
	--exclude=javy-test-invalid-plugin \
	--exclude=javy-test-plugin-wasip1 \
	--exclude=javy-test-plugin-wasip2 \
	--release --all-targets --all-features -- -D warnings

test-native-targets: build-default-plugin build-test-plugins test-native-targets-ci

# This command assumes a CI environment in which the test plugin
# assets have been previously created in the expected directories.
# This ensures that we can recycle CI time.
test-native-targets-ci:
	CARGO_PROFILE_RELEASE_LTO=off cargo hack test --workspace \
	--exclude=javy \
	--exclude=javy-plugin-api \
	--exclude=javy-plugin \
	--exclude=javy-test-invalid-plugin \
	--exclude=javy-test-plugin-wasip1 \
	--exclude=javy-test-plugin-wasip2 \
	--release --each-feature -- --nocapture


native-targets: lint-native-targets test-native-targets

# === Web Platform Tests

# For usage in CI, in which we assume pre-existing assets.
test-wpt-ci:
	npm install --prefix wpt
	npm test --prefix wpt

test-wpt: cli test-wpt-ci

# === All tests ===
test-all: wasi-targets native-targets test-wpt


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

# First, build the default plugin, which is a dependency to the CLI.
# No need to run `javy_plugin_processing`, the CLI build.rs will take
# care of doing that.
cli: build-default-plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo build -p=javy-cli --release

# Build the default plugin
build-default-plugin:
	cargo build -p=javy-plugin --target=wasm32-wasip1 --release

# Build auxiliary plugins, for testing
build-test-plugins:
	cargo build --package=javy-test-plugin-wasip1 --target=wasm32-wasip1 --release
	cargo build --package=javy-test-plugin-wasip2 --target=wasm32-wasip2 --release
	cargo build --package=javy-test-invalid-plugin --target=wasm32-unknown-unknown --release
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip1/release/test_plugin.wasm target/wasm32-wasip1/release/test_plugin.wasm
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip2/release/test_plugin.wasm target/wasm32-wasip2/release/test_plugin.wasm
