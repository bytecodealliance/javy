.PHONY: fmt fmt-check lint-wasi-targets lint-wasip1-targets lint-wasip2-targets \
	test-wasi-targets test-wasip1-targets test-wasip2-targets wasi-targets \
	lint-native-targets lint-native-targets-ci test-native-targets \
	test-native-targets-ci native-targets test-wpt test-wpt-ci test-all \
	clean cli build-default-plugin build-requested-feature-assets \
	build-all-feature-assets build-test-plugins ci
.DEFAULT_GOAL := cli

# Selection for linting / testing.
#
# Definitions are expected in`[package.metadata.javy]` in the
# corresponding Cargo.toml:
#
#   targets = ["native" | "wasip1" | "wasip2"]
#
# Note that if no targets are defined, the default is [ "native" ]
#
# Whether a crate has tests is read from cargo's own per-target `test`
# flag (e.g. `[lib] test = false`), so test-only fixtures opt out via
# the standard cargo manifest rather than custom metadata.  The
# auxiliary plugin crates (javy-test-plugin-wasi{p1, p2}) are
# currently the only crates exempt from unit/integration testing.
#
# Expected arguments:
#   1: target tag,
#   2: "lint" or "test"
define select_crates_for
$(shell cargo metadata --format-version=1 --no-deps | jq -r \
  --arg tgt $(1) --arg mode $(2) '\
    .packages[] \
    | (.metadata.javy.targets // ["native"]) as $$tgts \
    | (.targets | any(.test)) as $$has_tests \
    | select($$tgts | any(. == $$tgt)) \
    | select($$mode == "lint" or $$has_tests) \
    | "-p \(.name)"')
endef

NATIVE_LINT_CRATES := $(call select_crates_for,native,lint)
NATIVE_TEST_CRATES := $(call select_crates_for,native,test)
WASIP1_LINT_CRATES := $(call select_crates_for,wasip1,lint)
WASIP1_TEST_CRATES := $(call select_crates_for,wasip1,test)
WASIP2_LINT_CRATES := $(call select_crates_for,wasip2,lint)
WASIP2_TEST_CRATES := $(call select_crates_for,wasip2,test)

# Building optional feature assets
#
# In stable Rust, it is impossible to specify binary dependencies (see
# https://rust-lang.github.io/rfcs/3028-cargo-binary-dependencies.html).
# This function queries through the CLI metadata, the command(s)
# needed to build asset dependencies belonging to any of the specified
# features, e.g.,
#
#     make cli features=profiler
#
# it will retrive `cargo build -p javy-profiler-lib
# --target=wasm32-wasip1 --release` as defined the CLI's Cargo.toml
define feature_asset_cmds
$(shell cargo metadata --format-version=1 --no-deps | jq -r \
  --arg feats "$(1)" '\
    .packages[] \
    | select(.name == "javy-cli") \
    | (.metadata.javy."feature-assets" // {}) as $$m \
    | ($$feats | gsub(","; " ") | split(" ") | map(select(length > 0))) as $$req \
    | [ $$m | to_entries[] \
        | select($$feats == "all" or (.key as $$k | $$req | index($$k))) \
        | .value | if type == "array" then .[] else . end ] \
    | join(" && ")')
endef

cargo_features = $(if $(strip $(features)),--features "$(features)")

# === Format checks ===
fmt-check:
	cargo fmt --all --check
fmt:
	cargo fmt --all

# === Lint & Test WASI Targets ===
lint-wasi-targets: fmt-check lint-wasip1-targets lint-wasip2-targets

lint-wasip1-targets:
	cargo clippy $(WASIP1_LINT_CRATES) \
	--target=wasm32-wasip1 --all-targets --all-features -- -D warnings

lint-wasip2-targets:
	cargo clippy $(WASIP2_LINT_CRATES) \
	--target=wasm32-wasip2 --all-targets --all-features -- -D warnings

test-wasi-targets: test-wasip1-targets test-wasip2-targets

test-wasip1-targets:
	cargo hack test $(WASIP1_TEST_CRATES) \
	--target=wasm32-wasip1 --each-feature -- --nocapture

test-wasip2-targets:
	cargo hack test $(WASIP2_TEST_CRATES) \
	--target=wasm32-wasip2 --each-feature -- --nocapture

wasi-targets: lint-wasi-targets test-wasi-targets

# === Lint & Test Native Targets ===
lint-native-targets: build-default-plugin build-all-feature-assets lint-native-targets-ci

lint-native-targets-ci: fmt-check
	CARGO_PROFILE_RELEASE_LTO=off cargo clippy $(NATIVE_LINT_CRATES) \
	--release --all-targets --all-features -- -D warnings

test-native-targets: build-default-plugin build-all-feature-assets build-test-plugins test-native-targets-ci

# This command assumes a CI environment in which the test plugin
# assets have been previously created in the expected directories.
# This ensures that we can recycle CI time.
test-native-targets-ci:
	CARGO_PROFILE_RELEASE_LTO=off cargo hack test $(NATIVE_TEST_CRATES) \
	--release --each-feature -- --nocapture


native-targets: lint-native-targets test-native-targets

# === Web Platform Tests

# For usage in CI, in which we assume pre-existing assets.
test-wpt-ci:
	pnpm -C wpt install
	pnpm -C wpt test

test-wpt: cli test-wpt-ci

# === All tests ===
test-all: wasi-targets native-targets test-wpt


# === Misc ===
clean:
	cargo clean

# Intended to simulate what the GitHub Actions CI workflow will run.
# We don't invoke this directly because we often run out of disk space in
# GitHub Actions if we try to compile native targets in the same workflow as
# WASI targets so we have to use a multi-step process in GitHub to avoid that.
ci: lint-wasi-targets lint-native-targets test-all

# First, build the default plugin, which is a dependency to the CLI.
# No need to run `javy_plugin_processing`, the CLI build.rs will take
# care of doing that.
#
# Any optional features are excluded from the default build;
# use `make cli features=<list>` to enable them.
cli: build-default-plugin build-requested-feature-assets
	CARGO_PROFILE_RELEASE_LTO=off cargo build -p=javy-cli --release $(cargo_features)

# Build the default plugin
build-default-plugin:
	cargo build -p=javy-plugin --target=wasm32-wasip1 --release

# Build the assets for the requested features via `features=<list>`.
build-requested-feature-assets:
	$(call feature_asset_cmds,$(features))

# Build all feature assets, this is useful for testing/liting since
# all features will be enabled.
build-all-feature-assets:
	$(call feature_asset_cmds,all)

# Build auxiliary plugins, for testing
build-test-plugins:
	cargo build --package=javy-test-plugin-wasip1 --target=wasm32-wasip1 --release
	cargo build --package=javy-test-plugin-wasip2 --target=wasm32-wasip2 --release
	cargo build --package=javy-test-invalid-plugin --target=wasm32-unknown-unknown --release
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip1/release/test_plugin.wasm target/wasm32-wasip1/release/test_plugin.wasm
	cargo run --package=javy-plugin-processing --release -- target/wasm32-wasip2/release/test_plugin.wasm target/wasm32-wasip2/release/test_plugin.wasm
