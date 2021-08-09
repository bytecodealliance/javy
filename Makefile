.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

cli: core
		cd crates/cli && cargo build --release

core:
		cd crates/core && cargo build --release

tests: core
		cd crates/cli \
				&& cargo test --release \
				&& cargo check --benches --release

fmt:
		cd crates/quickjs-sys/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \
		&& cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \
		&& cd crates/cli/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean:
		cargo clean
