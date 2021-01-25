use std::fs;
use std::io::{Error, Result, Write};
use tokio::net::UnixListener;

pub struct X11Lock {
    display: u32,
}

impl Drop for X11Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(format!("/tmp/.X{}-lock", self.display));
    }
}

impl X11Lock {
    pub fn acquire(display: u32, force: bool) -> Result<Self> {
        let name = format!("/tmp/.X{}-lock", display);

        loop {
            // Try to create a new file. This will fail if the lock exists already.
            let file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&name);

            match file {
                Ok(mut file) => {
                    // Fresh file, just write our PID into it and we got the lock
                    match write!(file, "{:>10}\n", std::process::id()) {
                        Ok(_) => return Ok(X11Lock { display }),
                        Err(err) => {
                            let _ = fs::remove_file(&name);
                            return Err(err);
                        }
                    }
                }
                Err(_) => {
                    // A lock exists already. Try to see if the lock holder is still alive.
                    let content = fs::read_to_string(&name)?;
                    let pid = content.trim().parse::<libc::pid_t>().map_err(|_| {
                        Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("{} does not contain a valid PID", name),
                        )
                    })?;

                    let alive = unsafe { libc::kill(pid, 0) } == 0
                        || Error::last_os_error().raw_os_error().unwrap() as libc::c_int
                            != libc::ESRCH;

                    // The process is still alive
                    if alive && !force {
                        return Err(Error::new(
                            std::io::ErrorKind::AddrInUse,
                            format!("X{} is current in use", display),
                        ));
                    }

                    // The process is dead, remove the file and try again
                    std::fs::remove_file(&name)?;
                }
            }
        }
    }

    pub fn bind(&self) -> Result<UnixListener> {
        let name = format!("/tmp/.X11-unix/X{}", self.display);

        // Remove existing socket
        let _ = std::fs::create_dir_all("/tmp/.X11-unix");
        let _ = std::fs::remove_file(&name);

        UnixListener::bind(name)
    }
}
