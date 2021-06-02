WSLD: WSL Daemon
==============

![CI](https://github.com/nbdd0121/wsld/workflows/CI/badge.svg?branch=master)

Persist X11 connection when network changes or PC resumes from disconencted sleep/hibernation, and keep your WSL time in sync.

Formerly called x11-over-vsock; renamed to wsld as it gained extra functionality.

Implementation detail can be found [here](docs/impl.md).

## Build

This program is written in Rust. If you do not have Rust toolchain installed you can get it from https://rustup.rs/. Building on Windows also requires Visual C++ toolchain.

Install in WSL using `cargo install --git https://github.com/nbdd0121/wsld wsld` and install in Windows using `cargo install --git https://github.com/nbdd0121/wsld wsldhost` (The binary will be installed to `~/.cargo/bin/wsld` and `%USERPROFILE%\.cargo\bin\wsldhost.exe`).

You can also download pre-built binaries from [releases](https://github.com/nbdd0121/wsld/releases), which are automatically compiled using [GitHub Actions](https://github.com/nbdd0121/wsld/actions).

## Usage

In WSL, you will need to put config file `.wsld.toml` in your home directory. It should look like this:
```toml
# Leave out this section to disable X11 forwarding
[x11]
# X11 display number to listen *inside* WSL. The X server in Windows can specified as argument when running wsldhost.exe.
# Default to 0, can be omitted.
display = 0

# Leave out this section to disable time synchronisation
# If you need time synchronisation, you should either run wsld with root, or give it `cap_sys_time` capability using `sudo setcap cap_sys_time+eip <PATH to wsld>`.
[time]
# Interval between syncs
# Default to 10min, can be omitted
interval = "1hr"

# Leave out this section to disable TCP port forwarding
# This feature is experimental, feedbacks and suggestions welcome.
# This feature will WSL localhost to Windows localhost, so you can connect
# servers running in Windows in WSL. This feature requires interaction with
# iptables, so you need to either run wsld as root or allow wsld to use sudo.
[tcp_forward]
# iptables command to use. nftables seem not to work so you need to have
# the legacy one. If you are not using Debian it is likely you need to change it.
iptables_cmd = "sudo iptables-legacy"
# Ports to forward
ports = [ 1234 ]
```
then run `wsld` and set `DISPLAY=:0`.

In Windows, start a X server (e.g. VcXsrv) on TCP port 6000, and execute `wsldhost.exe --daemon` with administrator privilege. To know why administrator privilege is needed, check out [implementation detail](docs/impl.md). If your X server runs on a different port, you can add `--display localhost:<port>` to arguments.

To automatically start both services without manual intervention, see [here](docs/auto.md).

