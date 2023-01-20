.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

download-wasi-sdk:
	./install-wasi-sdk.sh

install:
	cargo install --path crates/cli

bench: cli
		cd crates/cli && cargo bench && cd -

cli: core
		cd crates/cli && cargo build --release && cd -

core:
		cd crates/core \
				&& cargo build --release --target=wasm32-wasi \
				&& wizer ../../target/wasm32-wasi/release/javy_quickjs_provider.wasm --allow-wasi -o ../../target/wasm32-wasi/release/javy_quickjs_provider_wizened.wasm \
				&& cd -

docs:
		cd crates/core \
				&& cargo doc --open --target=wasm32-wasi \
				&& cd -

test-quickjs-wasm-rs:
		cd crates/quickjs-wasm-rs \
				&& cargo wasi test --features json -- --nocapture \
				&& cd -

test-core:
		cd crates/core \
				&& cargo wasi test -- --nocapture \
				&& cd -

# Test in release mode to skip some debug assertions
# Note: to make this faster, the engine should be optimized beforehand (wasm-strip + wasm-opt).
test-cli: core
		cd crates/cli \
				&& cargo test --release -- --nocapture\
				&& cd -

test-wpt: cli
		cd wpt \
			&& npm install \
			&& npm test \
			&& cd -

tests: test-quickjs-wasm-rs test-core test-cli test-wpt

fmt: fmt-quickjs-wasm-sys fmt-quickjs-wasm-rs fmt-core fmt-cli

fmt-quickjs-wasm-sys:
		cd crates/quickjs-wasm-sys/ \
				&& cargo fmt -- --check \
				&& cargo clippy --target=wasm32-wasi -- -D warnings \
				&& cd -

fmt-quickjs-wasm-rs:
		cd crates/quickjs-wasm-rs/ \
				&& cargo fmt -- --check \
				&& cargo clippy --target=wasm32-wasi -- -D warnings \
				&& cd -

fmt-core:
		cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy --target=wasm32-wasi -- -D warnings \
				&& cd -

fmt-cli:
		cd crates/cli/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean: clean-wasi-sdk clean-cargo

clean-cargo:
		cargo clean

clean-wasi-sdk:
		rm -r crates/quickjs-wasm-sys/wasi-sdk 2> /dev/null || true
