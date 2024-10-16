.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

install:
	cargo install --path crates/cli

bench: cli
	cargo bench --package=javy-cli

check-bench:
	CARGO_PROFILE_RELEASE_LTO=off cargo check --package=javy-cli --release --benches

# Disabling LTO substantially improves compile time
cli: core
	CARGO_PROFILE_RELEASE_LTO=off cargo build --package=javy-cli --release

core:
	cargo build --package=javy-core --release --target=wasm32-wasip1 --features=$(CORE_FEATURES)

docs:
	cargo doc --package=javy-cli --open
	cargo doc --package=javy-core --open --target=wasm32-wasip1

test-javy:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo test --package=javy --target=wasm32-wasip1 --features json,messagepack -- --nocapture

test-core:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime" cargo wasi test --package=javy-core -- --nocapture

# Test in release mode to skip some debug assertions
# Note: to make this faster, the engine should be optimized beforehand (wasm-strip + wasm-opt).
# Disabling LTO substantially improves compile time
test-cli: core
	CARGO_PROFILE_RELEASE_LTO=off cargo test --package=javy-cli --release --features=$(CLI_FEATURES) -- --nocapture

test-runner:
	cargo test --package=javy-runner -- --nocapture

# WPT requires a Javy build with the experimental_event_loop feature to pass
test-wpt: export CORE_FEATURES ?= experimental_event_loop
test-wpt:
# Can't use a prerequisite here b/c a prequisite will not cause a rebuild of the CLI
	$(MAKE) cli
	npm install --prefix wpt
	npm test --prefix wpt 

test-config:
	CARGO_PROFILE_RELEASE_LTO=off cargo test --package=javy-config -- --nocapture

tests: test-javy test-core test-runner test-cli test-wpt test-config

fmt: fmt-javy fmt-core fmt-cli

fmt-javy:
	cargo fmt --package=javy -- --check
	cargo clippy --package=javy --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-core:
	cargo fmt --package=javy-core -- --check
	cargo clippy --package=javy-core --target=wasm32-wasip1 --all-targets -- -D warnings

# Use `--release` on CLI clippy to align with `test-cli`.
# This reduces the size of the target directory which improves CI stability.
fmt-cli:
	cargo fmt --package=javy-cli -- --check
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy --package=javy-cli --release --all-targets -- -D warnings
