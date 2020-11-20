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

In WSL, run `x11-over-vsock` and set `DISPLAY=:0`.

In Windows, start a X server (e.g. VcXsrv) on TCP port 6000, and you can either:
* Execute `hcsdiag list` with administrator privilege to get the VMID of your WSL instance, then `x11-over-vsock.exe <VMID>` (no administrator privilege required). WSL must be running before execution, and you will need to kill and start the process again if shutdown you shutdown the WSL utility VM.
* Execute `x11-over-vsock.exe` with administrator privilege. It will automatically retrieve WSL VMID. WSL must be running before execution, and you will need to kill and start the process again if you shutdown the WSL utility VM.
* Execute `x11-over-vsock.exe --daemon` with administrator privilege. It will poll WSL status every 5 seconds, and start/shutdown server automatically.

### Installation

Perform both these tasks - on Windows and inside WSL2 - to get a
fully-automatic setup that "does the right thing" even after reboot without
manual intervention:

*On Windows:*

* Copy `x11-over-vsock.exe` e.g. to
  `%USERPROFILE%\.cargo\bin\x11-over-vsock.exe` if you didn't build it
  yourself.
* Create a Scheduled Task to start `x11-over-vsock.exe` at login
    * Open "Task Scheduler"
    * Actions &rarr; "Create Task..."
    * General (tab): Run with highest privileges
    * Triggers (tab): New (button), Begin the task, At log on
    * Actions (tab): Start a program, Program/script =
      `%USERPROFILE%\.cargo\bin\x11-over-vsock.exe` (remember quotes if
      appropriate!),
      Add arguments = `--daemon`
    * Conditions (tab): Uncheck "Start the task only if the computer is on AC
      power" and "Stop if the computer switches to battery power"
    * Settings (tab): Uncheck "Stop the task if it runs longer than"

It should now start up at every boot as Administrator with the `--daemon`
option. Now either start `x11-over-vsock.exe` by right-click-ing on the newly
created task and clicking "Run" or reboot the Windows 10 installation to start
it.

*On Linux under WSL2:*

* Copy `x11-over-vsock` e.g. to `~/.cargo/bin/x11-over-vsock` if you didn't
  build it yourself.
* Make sure `xset` is in your path, e.g. with `sudo apt-get install
  x11-xserver-utils` in Ubuntu.
* Add this to your `~/.bashrc` or `~/.zshrc`:

``` bash
export DISPLAY=:0

if ! pgrep x11-over-vsock >> /dev/null 2>&1 ; then
    nohup ~/.cargo/bin/x11-over-vsock > /dev/null < /dev/null 2>&1 &
    disown

    # sleep until $DISPLAY is up
    while ! xset q > /dev/null 2>&1 ; do
        sleep 0.3
    done
fi
```

Using `xset q` to test the `$DISPLAY` makes it possible to run a command like:

    C:\Windows\System32\wsl.exe bash --login -c some-terminal

Otherwise `some-terminal` will fail because the `$DISPLAY` isn't ready yet...
