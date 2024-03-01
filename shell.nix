#!/usr/bin/env nix-shell

with import <nixpkgs> { };
mkShell {
  name = "xdp-router";

  RUST_BACKTRACE = 0;

  buildInputs = [
    rustup
    cargo-watch

    xdp-tools
    bpftool
  ];

  shellHook = ''
    if ! cargo version 2> /dev/null; then
      echo "Installing rustup"
      rustup default stable
      rustup toolchain install nightly

      rustup component add rust-src
      rustup component add rust-analyzer
      rustup component add cargo-deb

      cargo install bpf-linker
      # export PATH=$PATH:$HOME/.nix-profile/bin
    fi

    echo ">>>>> $name"
    echo -n ">>>>> "; gcc   --version | head -n1 | awk '{print $1"  ", $3, $2}'
    echo -n ">>>>> "; cargo --version
  '';
}
