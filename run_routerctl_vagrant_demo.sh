#!/usr/bin/env bash

set -xeuo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)

cd "$ROOT"/target/x86_64-unknown-linux-musl/debug

./routerctl 192.168.171.10:6707 set policy drop
./routerctl 192.168.171.10:6707 add mirror 12345

echo "Type anything and press Enter"
nc -u 192.168.171.10 12345
