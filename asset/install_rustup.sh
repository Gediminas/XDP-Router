#!/usr/bin/env bash

set -xeuo pipefail

echo "=============================================="
echo "Installing RUST..."
id

sudo apt-get install -y curl git build-essential

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install_rustup.sh
sh /tmp/install_rustup.sh -y
source "$HOME/.cargo/env"
