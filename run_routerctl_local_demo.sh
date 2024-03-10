#!/usr/bin/env bash

set -xeuo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)

cd "$ROOT"

cargo run --bin routerctl -- 127.0.0.1:6707 set policy accept
cargo run --bin routerctl -- 127.0.0.1:6707 add mirror 12345

echo "Type anything and press Enter"
nc -u 127.0.0.1 12345
