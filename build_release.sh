#!/usr/bin/env bash

set -euo pipefail

cargo xtask build-ebpf --release
cargo build --release
