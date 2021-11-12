.PHONY: cli core test fmt clean core-wasi cli-wasi
.DEFAULT_GOAL := cli

install:
	cargo install --path crates/cli

cli: core
		cd crates/cli && cargo build --release && cd -

cli-wasi: core-wasi
		cd crates/cli && cargo build --release && cd -

check-benchmarks:
		cd crates/benchmarks \
				&& cargo check --benches --release \
				&& cd -

core:
		cd crates/core \
				&& cargo build --release --target=wasm32-wasi \
				&& cd -

core-wasi:
		cd crates/core \
			&& cargo build --release --target=wasm32-wasi --features standalone-wasi \
			&& cd -

wizen-core:
	wizer target/wasm32-wasi/release/javy_core.wasm -o crates/cli/javy_core.wizened.wasm 

test-core:
		cd crates/core \
				&& cargo wasi test --features standalone-wasi -- --nocapture \
				&& cd -

# Test in release mode to skip some debug assertions
# Note: to make this faster, the engine should be optimized beforehand (wasm-strip + wasm-opt).
test-cli: core
		cd crates/cli \
				&& cargo test --release \
				&& cd -

tests: check-benchmarks test-core test-cli

fmt: fmt-quickjs-sys fmt-core fmt-cli

fmt-quickjs-sys:
		cd crates/quickjs-sys/ \
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

fmt-binaries:
		cd crates/binaries/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean:
		cargo clean
