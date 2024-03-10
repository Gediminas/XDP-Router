#!/usr/bin/env bash

set -xeuo pipefail

echo "=============================================="
echo "Installing SERVER..."
id

apt-get update
apt-get install -y tcpdump ethtool xdp-tools bpftool

# Suppress SSH login message
touch /home/vagrant/.hushlogin

# Go to project folder when SSH'ed
echo 'cd /vagrant' >> "/home/vagrant/.bashrc"
echo 'cd /vagrant' >> "/root/.bashrc"
