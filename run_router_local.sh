#!/usr/bin/env bash

set -xeuo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)

cd "$ROOT"

sudo -E cargo run --bin router -- --iface lo --bind 127.0.0.1:6707 --log-level trace
