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
    if ! cargo version 2> /dev/null || [ ! -f ".prepared_rustup" ]; then
      echo "Installing rustup"
      rustup default stable
      rustup target add x86_64-unknown-linux-gnu
      rustup target add x86_64-unknown-linux-musl
      rustup toolchain install nightly
      rustup component add rust-src
      rustup component add rust-analyzer
      touch .prepared_rustup
    fi

    echo ">>>>> $name"
    echo -n ">>>>> "; gcc   --version | head -n1 | awk '{print $1"  ", $3, $2}'
    pushd router-xdp >/dev/null
    echo -n ">>>>> "; cargo --version
    popd >/dev/null
    echo -n ">>>>> "; cargo --version
  '';
}
