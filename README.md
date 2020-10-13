X11 over Vsock
==============

![CI](https://github.com/nbdd0121/x11-over-vsock/workflows/CI/badge.svg?branch=master)

## Background

Windows will reset all external TCP connections when network changes or when PC resumes from disconencted sleep/hibernation, which include connections on the WSL bridge. If you are using X11, this can be annoying because all X11 connections over TCP will also drop.

## Solution

Unlike TCP connections, Vsock connections will not be dropped. Vsock is VM socket for communication between the guest VM and the host, mostly used to provide [integration service](https://docs.microsoft.com/en-us/virtualization/hyper-v-on-windows/user-guide/make-integration-service). In WSL2, Vsock is used for many interops (e.g. file/network/executable). This program is just another integration service.

Two executables are to be ran, one inside WSL2 and another outside. The program inside WSL2 will listen on Unix socket /tmp/.X11-unix/X0 (DISPLAY=:0) and forward it the program outside WSL2 via Vsock. The program outside WSL2 will listen on the Vsock and forward it to TCP port 6000 to which your X server should listen.

## Build

This program is written in Rust. If you do not have Rust toolchain installed you can get it from https://rustup.rs/.

Install in both WSL and Windows using `cargo install --git https://github.com/nbdd0121/x11-over-vsock` (The binary will be installed to `~/.cargo/bin/x11-over-vsock` and `%USERPROFILE%\.cargo\bin\x11-over-vsock.exe`).

You can also download pre-built binaries from [GitHub Actions artifacts](https://github.com/nbdd0121/x11-over-vsock/actions).

## Usage

First, execute `hcsdiag list` with administrator privilege to get the VMID of your WSL instance. Then, run `x11-over-vsock.exe <VMID>` on Windows, and run `x11-over-vsock` in WSL. Set `DISPLAY=:0` inside WSL and start a X server (e.g. VcXsrv) on TCP port 6000 inside Windows.
