.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

cli: core
		cd crates/cli && cargo build --release

check-benchmarks:
	cd crates/benchmarks \
		&& cargo check --benches --release

core:
		cd crates/core \
			&& cargo build --release --target=wasm32-wasi

tests: check-benchmarks core

fmt: fmt-quickjs-sys fmt-core fmt-cli

fmt-quickjs-sys:
		cd crates/quickjs-sys/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \

fmt-core:
		cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \

fmt-cli:
		cd crates/cli/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean:
		cargo clean
