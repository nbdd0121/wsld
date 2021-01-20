Automatic Startup
==============

Perform both these tasks - on Windows and inside WSL2 - to get a
fully-automatic setup that "does the right thing" even after reboot without
manual intervention:

## On Windows

Create a Scheduled Task to start `x11-over-vsock-server.exe` at login
* Open `Task Scheduler`
* Actions &rarr; `Create Task...`
* General (tab): Check `Run with highest privileges`
* Triggers (tab): Click `New`, select `At log on` under `Begin the task`.
* Actions (tab): Click, `New`, select `Start a program` under `Action`, set `Program/script` to the path of `x11-over-vsock-server.exe` wherever it is placed, set `Add arguments` to `--daemon`.
* Conditions (tab): Uncheck `Start the task only if the computer is on AC power` and `Stop if the computer switches to battery power`
* Settings (tab): Uncheck `Stop the task if it runs longer than`

It should now start up at every boot as Administrator with the `--daemon`
option. Now start `x11-over-vsock-server.exe` by right-click-ing on the newly
created task and clicking `Run`.

## On WSL2

* Make sure `xset` is installed, e.g. with `sudo apt-get install
  x11-xserver-utils` in Debian-based distributions.
* Add this to your `~/.profile` or `~/.bash_profile` or `~/.zlogin`:

``` bash
export DISPLAY=:0

if ! pgrep x11-over-vsock-client >> /dev/null 2>&1 ; then
    nohup x11-over-vsock-client > /dev/null < /dev/null 2>&1 &
    disown

    # sleep until $DISPLAY is up
    while ! xset q > /dev/null 2>&1 ; do
        sleep 0.3
    done
fi
```

Using `xset q` to test the `$DISPLAY` makes it possible to run a command like `wsl.exe bash --login -c some-terminal`, otherwise `some-terminal` will fail because the `$DISPLAY` isn't ready yet.
