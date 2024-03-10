#!/usr/bin/env bash

set -xeuo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)

cd "$ROOT"

echo "=============================================="
echo "Preparing RUST..."
id

rustup default stable
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup component add rust-src
rustup component add rust-analyzer
rustup toolchain install nightly
cargo install bpf-linker
