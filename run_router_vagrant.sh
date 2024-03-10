#!/usr/bin/env bash

set -xeuo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)

cd "$ROOT"/target/x86_64-unknown-linux-musl/debug

sudo ./router --iface eth1 --bind 0.0.0.0:6707 --log-level trace

