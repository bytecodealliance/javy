.PHONY: core cli all

all: core cli

core:
ifeq ($(profile),release)
		cd crates/core && cargo build --release
else
		cd crates/core && cargo build
endif

cli:
ifeq ($(profile),release)
		cd crates/cli && cargo build --release
else
		cd crates/cli && cargo build
endif
