Implementation Detail
=====================

## X11 Disconnection Problem

Windows will reset all external TCP connections when network changes or when PC resumes from disconencted sleep/hibernation, which include connections on the WSL bridge. If you are using X11, this can be annoying because all X11 connections over TCP will also drop.

## Hyper-V Integration Service

Unlike TCP connections, Vsock connections will not be dropped. Vsock is VM socket for communication between the guest VM and the host, mostly used to provide [integration service](https://docs.microsoft.com/en-us/virtualization/hyper-v-on-windows/user-guide/make-integration-service). In WSL2, Vsock is used for many interops (e.g. file/network/executable). This program is just another integration service.

Two executables are to be ran, one inside WSL2 and another outside. The program inside WSL2 will listen on Unix socket /tmp/.X11-unix/X0 (DISPLAY=:0) and forward it the program outside WSL2 via Vsock. The program outside WSL2 will listen on the Vsock and forward it to TCP port 6000 to which your X server should listen.

## Finding the VM

WSL utility VM is not a regular Hyper-V VM. It will not show up in Hyper-V Manager, and normal Hyper-V API and commands do not apply to the WSL VM. Integration service that listens on wildcard VMID will not accept connections from WSL VM.

The only way around is to retrieve WSL VM's VMID and listen on that specific address. User can use `hcsdiag list` to obtain the VMID, while we use a semi-documented API called `HcsEnumerateComputeSystems` to obtain the VMID. Both methods require administrator privilege.

The WSL utility VM is created dynamically, so it does not exist (nor does its VMID) when WSL is not running. The VM gets destroyed when WSL shuts down, and its VMID will change when WSL is launched next time. So the daemon mode will poll WSL status every 5 seconds, and start/shutdown server accordingly.

`x11-over-vsock-server` can be launched with a specific VMID, if you prefer to find VMID woth `hcsdiag list` yourself or use a Hyper-V VM's VMID.

## Clock Skew Problem

WSL clocks can go out of sync with the host, see https://github.com/microsoft/WSL/issues/4677. Hyper-V does have time synchronisation integration service, but as mentioned in [Finding the VM](#finding-the-vm), WSL VM is not a normal Hyper-V VM and therefore it is likely this time synchronisation service does not apply to WSL.

We basically just implement the integration service ourselves.