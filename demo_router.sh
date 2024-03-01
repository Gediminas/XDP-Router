#!/usr/bin/env bash

set -euo pipefail

sudo -E cargo run --bin router -- --iface lo --bind 127.0.0.1:6707 --log-level trace
