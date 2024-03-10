# XDP-Router

## Description

A **Rust** toy/demo project utilizing **eBPF XDP** for high-performance packet routing at the network driver level.

The project is composed of three main components:
- **router_xdp**: The core eBPF XDP program handling packet routing
- **router**: A server application that loads the XDP program, manages XDP routing maps, and listens for TCP commands
- **routerctl**: A TCP client for managing the server

UDP packet **mirroring/pong** demo:
![demo1](./doc/demo_mirroring.png)

UDP packet **routing** demo:
![demo1](./doc/demo_routing.png)

## Build and Run locally

- Install:
  - nix: `nix-shell`
  - nix+direnv: `direnv allow .`
  - Debian / Ubuntu:
    - install rustup: `./asset/install_rustup.sh`
    - prepare rustup: `./asset/prepare_rustup.sh`
  - other:
    - install rustup: https://www.rust-lang.org/tools/install  
                      `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - prepare rustup: `./asset/prepare_rustup.sh`

- Build:
  ```bash
  ./build_debug.sh
  ```

- Run server, load XDP:
  ```bash
  ./run_router_local.sh
  ```

- Run client:
  ```bash
  ./run_routerctl_local_demo.sh
  ```


## Build and Run on Vagrant

- Start Debian 12 "Bookworm" VMs:
  ```sh
  vagrant up
  ```

- Build, run server, load XDP (on VM):
  ```sh
  vagrant ssh server --command "/vagrant/build_debug.sh"
  vagrant ssh server --command "/vagrant/run_router_vagrant.sh"
  ```

- Run client (optionaly on VM):
  ```sh
  vagrant ssh client --command "/vagrant/run_routerctl_vagrant_demo.sh"
  # or just: ./run_routerctl_vagrant_demo.sh
  ```


## Troubleshooting

Check if XDP was loaded:

  ```sh
  vagrant ssh server
  sudo xdp-loader status | grep router_xdp
  sudo bpftool prog show | grep router_xdp
  ```

Test mirroring/pong with debug port:
  ```sh
  # XDP loaded locally
  nc -u 127.0.0.1 65500

  # XDP loaded on vagrant
  nc -u 192.168.171.10 65500
  ```
