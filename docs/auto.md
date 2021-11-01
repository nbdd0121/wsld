Automatic Startup
==============

Perform both these tasks - on Windows and inside WSL2 - to get a
fully-automatic setup that "does the right thing" even after reboot without
manual intervention:

## On Windows

Create a Scheduled Task to start `wsldhost.exe` at login
* Open `Task Scheduler`
* Actions &rarr; `Create Task...`
* General (tab): Check `Run with highest privileges`
* Triggers (tab): Click `New`, select `At log on` under `Begin the task`.
* Actions (tab): Click, `New`, select `Start a program` under `Action`, set `Program/script` to the path of `wsldhost.exe` wherever it is placed, set `Add arguments` to `--daemon`.
* Conditions (tab): Uncheck `Start the task only if the computer is on AC power` and `Stop if the computer switches to battery power`
* Settings (tab): Uncheck `Stop the task if it runs longer than`

It should now start up at every boot as Administrator with the `--daemon`
option. Now start `wsldhost.exe` by right-click-ing on the newly
created task and clicking `Run`.

## On WSL2

* Make sure `xset` is installed, e.g. with `sudo apt-get install
  x11-xserver-utils` in Debian-based distributions.
* Add this to your `~/.profile` or `~/.bash_profile` or `~/.zlogin`:

``` bash
function _wsl_x11_vsock() {
  # https://github.com/nbdd0121/wsld/blob/master/docs/auto.md#on-wsl2
  export DISPLAY=:0
  if ! pgrep wsld >> /dev/null 2>&1 ; then
    # https://github.com/nbdd0121/wsld/commit/c3a2bb7ccab8c11710fa6b49cf10434a80a853dd
    # Delete lock files/directories.
    rm -rf "/tmp/.X${DISPLAY/:/}-lock" "/tmp/.X11-unix/X${DISPLAY/:/}"
    nohup wsld > /dev/null < /dev/null 2>&1 &
    disown

    # sleep for N seconds until $DISPLAY is up
    local START;
    START=$(date +%s)
    local CURRENT=$START
    local WAIT=5
    while [[ $((CURRENT - START)) -le "$WAIT" ]]; do
      { xset q > /dev/null 2>&1 && break; } || sleep 0.3;
      CURRENT=$(date +%s)
    done
    if [[ $((CURRENT - START)) -gt "$WAIT" ]]; then
      echo >&2 "Please ensure that 'wsldhost.exe --daemon' and a X11 server is running on the Windows host."
      # Kill the current instance so it can be re-initialized.
      pkill wsld
    fi
  fi
}

# Initialize wsld
_wsl_x11_vsock
```

Using `xset q` to test the `$DISPLAY` makes it possible to run a command like `wsl.exe bash --login -c some-terminal`, otherwise `some-terminal` will fail because the `$DISPLAY` isn't ready yet.
