# XDP-Router

## Description

A **Rust** toy/demo project utilizing **eBPF XDP** for high-performance packet routing at the network driver level. Currently, UDP packet mirroring/pong implemented only.

The project is composed of three main components:
- **router_xdp**: The core eBPF XDP program handling packet routing
- **router**: A server application that loads the XDP program, manages XDP routing maps, and listens for TCP commands
- **routerctl**: A TCP client for managing the server


## Build 

```sh
./build_debug.sh
./build_release.sh
```


## Run 

```sh
./demo_router.sh
./demo_routerctl.sh
```


# Troubleshooting

Check if XDP was loaded:

```sh
./watch_xdp.sh
sudo xdp-loader status | grep router_xdp
sudo bpftool prog show | grep router_xdp
```

Test mirroring/pong with debug port:
nc -u 127.0.0.1 65500
