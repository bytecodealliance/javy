.PHONY: fmt fmt-check lint-wasi-targets test-wasi-targets wasi-targets lint-native-targets test-native-targets native-targets test-wpt test clean cli plugin build-test-plugins
.DEFAULT_GOAL := cli

# === Format checks ===
fmt-check:
	cargo fmt --all --check
fmt:
	cargo fmt --all

# === Lint & Test WASI Targets ===
lint-wasi-targets:
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
lint-native-targets:
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy --workspace \
	--exclude=javy \
	--exclude=javy-plugin-api \
	--exclude=javy-plugin \
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

# First, build the default plugin, which is a dependency to the CLI.
# No need to run `javy_plugin_processing`, the CLI build.rs will take
# care of doing that.
cli: build-default-plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo build -p=javy-cli --release

# Build the default plugin
build-default-plugin:
	cargo build -p=javy-plugin --target=wasm32-wasip2 --release

# Build auxiliary plugins, for testing
build-test-plugins:
	cargo build --package=javy-test-plugin-wasip2 --target=wasm32-wasip2 --release
	cargo build --package=javy-test-invalid-plugin --target=wasm32-unknown-unknown --release
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip2/release/test_plugin.wasm target/wasm32-wasip2/release/test_plugin.wasm
