X11 over Vsock
==============

![CI](https://github.com/nbdd0121/x11-over-vsock/workflows/CI/badge.svg?branch=master)

Persist X11 connection when network changes or PC resumes from disconencted sleep/hibernation, and keep your WSL time in sync.

Implementaion detail can be found [here](docs/impl.md).

## Build

This program is written in Rust. If you do not have Rust toolchain installed you can get it from https://rustup.rs/. Building on Windows also requires Visual C++ toolchain.

Install in WSL using `cargo install --git https://github.com/nbdd0121/x11-over-vsock x11-over-vsock-client` and install in Windows using `cargo install --git https://github.com/nbdd0121/x11-over-vsock x11-over-vsock-server` (The binary will be installed to `~/.cargo/bin/x11-over-vsock-client` and `%USERPROFILE%\.cargo\bin\x11-over-vsock-server.exe`).

You can also download pre-built binaries from [GitHub Actions artifacts](https://github.com/nbdd0121/x11-over-vsock/actions?query=branch%3Amaster).

## Usage

In WSL, you will need to put config file `.wsld.toml` in your home directory. It should look like this:
```toml
# Leave out this section to disable X11 forwarding
[x11]
# X11 display number to listen *inside* WSL. The X server in Windows currently is fixed to be on port 6000.
# Default to 0, can be omitted.
display = 0

# Leave out this section to disable time synchronisation
[time]
# Interval between syncs
# Default to 10min, can be omitted
interval = 1hr
```
then run `x11-over-vsock-client` and set `DISPLAY=:0`.

In Windows, start a X server (e.g. VcXsrv) on TCP port 6000, and execute `x11-over-vsock-server.exe --daemon` with administrator privilege. To know why administrator privilege is needed, check out [implementation detail](docs/impl.md).

To automatically start both services without manual intervention, see [here](docs/auto.md).

