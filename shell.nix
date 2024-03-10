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
      ./asset/prepare_rustup.sh
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
