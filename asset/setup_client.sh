#!/usr/bin/env bash

set -xeuo pipefail

echo "Installing CLIENT..."
echo "=============================================="
id

apt-get update
apt-get install -y tcpdump hping3

# Suppress SSH login message
touch /home/vagrant/.hushlogin

# Go to project compiled binaries folder when SSH'ed
echo 'cd /vagrant/target/x86_64-unknown-linux-musl/debug' >> "/home/vagrant/.bashrc"

