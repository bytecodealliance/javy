.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

download-wasi-sdk:
	./install-wasi-sdk.sh

install:
	cargo install --path crates/cli

bench: cli
	cargo bench --package=javy-cli

check-bench:
	cargo check --package=javy-cli --release --benches

cli: core
	cargo build --package=javy-cli --release

core:
	cargo build --package=javy-core --release --target=wasm32-wasi --features=$(CORE_FEATURES)

docs:
	cargo doc --package=javy-cli --open
	cargo doc --package=javy-core --open --target=wasm32-wasi

test-quickjs-wasm-rs:
	cargo wasi test --package=quickjs-wasm-rs -- --nocapture

test-javy:
	cargo wasi test --package=javy --features json,messagepack -- --nocapture

test-apis:
	cargo hack wasi test --package=javy-apis --each-feature -- --nocapture

test-core:
	cargo wasi test --package=javy-core -- --nocapture

# Test in release mode to skip some debug assertions
# Note: to make this faster, the engine should be optimized beforehand (wasm-strip + wasm-opt).
test-cli: core
	cargo test --package=javy-cli --release --features=$(CLI_FEATURES) -- --nocapture

# WPT requires a Javy build with the experimental_event_loop feature to pass
test-wpt: export CORE_FEATURES ?= experimental_event_loop
test-wpt:
# Can't use a prerequisite here b/c a prequisite will not cause a rebuild of the CLI
	$(MAKE) cli
	npm install --prefix wpt
	npm test --prefix wpt 

tests: test-quickjs-wasm-rs test-javy test-apis test-core test-cli test-wpt

fmt: fmt-quickjs-wasm-sys fmt-quickjs-wasm-rs fmt-javy fmt-apis fmt-core fmt-cli

fmt-quickjs-wasm-sys:
	cargo fmt --package=quickjs-wasm-sys -- --check
	cargo clippy --package=quickjs-wasm-sys --target=wasm32-wasi --all-targets -- -D warnings

fmt-quickjs-wasm-rs:
	cargo fmt --package=quickjs-wasm-rs -- --check
	cargo clippy --package=quickjs-wasm-rs --target=wasm32-wasi --all-targets -- -D warnings

fmt-javy:
	cargo fmt --package=javy -- --check
	cargo clippy --package=javy --target=wasm32-wasi --all-targets -- -D warnings

fmt-apis:
	cargo fmt --package=javy-apis -- --check
	cargo clippy --package=javy-apis --all-features --target=wasm32-wasi --all-targets -- -D warnings

fmt-core:
	cargo fmt --package=javy-core -- --check
	cargo clippy --package=javy-core --target=wasm32-wasi --all-targets -- -D warnings

# Use `--release` on CLI clippy to align with `test-cli`.
# This reduces the size of the target directory which improves CI stability.
fmt-cli:
	cargo fmt --package=javy-cli -- --check
	cargo clippy --package=javy-cli --release --all-targets -- -D warnings

clean: clean-wasi-sdk clean-cargo

clean-cargo:
	cargo clean

clean-wasi-sdk:
	rm -r crates/quickjs-wasm-sys/wasi-sdk 2> /dev/null || true
