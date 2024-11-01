.PHONY: cli plugin test fmt clean
.DEFAULT_GOAL := cli

install:
	cargo install --path crates/cli

bench: cli
	cargo bench --package=javy-cli

check-bench:
	CARGO_PROFILE_RELEASE_LTO=off cargo check --package=javy-cli --release --benches

# Disabling LTO substantially improves compile time
cli: plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo build --package=javy-cli --release

plugin:
	cargo build --package=javy-plugin --release --target=wasm32-wasip1 --features=$(PLUGIN_FEATURES)

docs:
	cargo doc --package=javy-cli --open
	cargo doc --package=javy-plugin --open --target=wasm32-wasip1

test-javy:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo hack test --package=javy --target=wasm32-wasip1 --each-feature -- --nocapture

test-plugin-api:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo hack test --package=javy-plugin-api --target=wasm32-wasip1 --each-feature -- --nocapture

test-plugin:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime" cargo test --package=javy-plugin --target=wasm32-wasip1 -- --nocapture

# Test in release mode to skip some debug assertions
# Note: to make this faster, the engine should be optimized beforehand (wasm-strip + wasm-opt).
# Disabling LTO substantially improves compile time
test-cli: plugin
	CARGO_PROFILE_RELEASE_LTO=off cargo test --package=javy-cli --release --features=$(CLI_FEATURES) -- --nocapture

test-runner:
	cargo test --package=javy-runner -- --nocapture

# WPT requires a Javy build with the experimental_event_loop feature to pass
test-wpt: export PLUGIN_FEATURES ?= experimental_event_loop
test-wpt:
# Can't use a prerequisite here b/c a prequisite will not cause a rebuild of the CLI
	$(MAKE) cli
	npm install --prefix wpt
	npm test --prefix wpt 

tests: test-javy test-plugin-api test-plugin test-runner test-cli test-wpt

fmt: fmt-javy fmt-plugin-api fmt-plugin fmt-cli

fmt-javy:
	cargo fmt --package=javy -- --check
	cargo clippy --package=javy --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-plugin-api:
	cargo fmt --package=javy-plugin-api -- --check
	cargo clippy --package=javy-plugin-api --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-plugin:
	cargo fmt --package=javy-plugin -- --check
	cargo clippy --package=javy-plugin --target=wasm32-wasip1 --all-targets -- -D warnings

# Use `--release` on CLI clippy to align with `test-cli`.
# This reduces the size of the target directory which improves CI stability.
fmt-cli:
	cargo fmt --package=javy-cli -- --check
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy --package=javy-cli --release --all-targets -- -D warnings
