#!/usr/bin/env bash

set -euo pipefail

cargo run --bin routerctl -- 127.0.0.1:6707 set policy accept
cargo run --bin routerctl -- 127.0.0.1:6707 add mirror 12345

echo "Type anything and press Enter"
nc -u 127.0.0.1 12345
