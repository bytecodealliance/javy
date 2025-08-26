.PHONY: cli plugin test fmt clean
.DEFAULT_GOAL := cli

install:
	cargo install --path crates/cli

bench: cli
	cargo bench --package=javy-cli

check-bench:
	cargo check --package=javy-cli --benches

cli: plugin
	cargo build --package=javy-cli

plugin:
	cargo build --package=javy-plugin --release --target=wasm32-wasip1 --features=$(PLUGIN_FEATURES)
	cargo run --package=javy-plugin-processing target/wasm32-wasip1/release/plugin.wasm target/wasm32-wasip1/release/plugin_wizened.wasm

build-test-plugin: cli
	cargo build --package=javy-test-plugin --release --target=wasm32-wasip1
	target/debug/javy init-plugin target/wasm32-wasip1/release/test_plugin.wasm -o crates/runner/test_plugin.wasm

docs:
	cargo doc --package=javy-cli --open
	cargo doc --package=javy-plugin --open --target=wasm32-wasip1

test-javy:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo hack test --package=javy --target=wasm32-wasip1 --each-feature -- --nocapture

test-plugin-api:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo hack test --package=javy-plugin-api --target=wasm32-wasip1 --each-feature -- --nocapture

test-plugin:
	CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime" cargo test --package=javy-plugin --target=wasm32-wasip1 -- --nocapture

test-plugin-processing:
	cargo test --package=javy-plugin-processing -- --nocapture

test-codegen: cli
	target/debug/javy emit-plugin -o crates/codegen/default_plugin.wasm
	cargo hack test --package=javy-codegen --each-feature -- --nocapture

test-cli: plugin build-test-plugin
	cargo test --package=javy-cli --features=$(CLI_FEATURES) -- --nocapture

test-runner:
	cargo test --package=javy-runner -- --nocapture

test-wpt: cli
	npm install --prefix wpt
	npm test --prefix wpt

tests: test-javy test-plugin-api test-plugin test-plugin-processing test-runner test-codegen test-cli test-wpt

fmt: fmt-javy fmt-plugin-api fmt-plugin fmt-plugin-processing fmt-cli fmt-codegen

fmt-javy:
	cargo fmt --package=javy -- --check
	cargo clippy --package=javy --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-plugin-api:
	cargo fmt --package=javy-plugin-api -- --check
	cargo clippy --package=javy-plugin-api --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-plugin:
	cargo fmt --package=javy-plugin -- --check
	cargo clippy --package=javy-plugin --target=wasm32-wasip1 --all-targets -- -D warnings

fmt-plugin-processing:
	cargo fmt --package=javy-plugin-processing -- --check
	cargo clippy --package=javy-plugin-processing --all-targets -- -D warnings

fmt-cli:
	cargo fmt --package=javy-cli -- --check
	cargo clippy --package=javy-cli --all-targets -- -D warnings

fmt-codegen:
	cargo fmt --package=javy-codegen -- --check
	cargo clippy --package=javy-codegen --all-targets -- -D warnings
