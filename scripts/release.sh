#!/usr/bin/env bash

set -euo pipefail

PACKAGES=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].name')

echo "$PACKAGES" | xargs cargo run --release -p javy-release -- $1
