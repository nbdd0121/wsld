// Hide console window in Windows
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod vmcompute;

#[cfg(windows)]
#[path = "vmsocket.windows.rs"]
mod vmsocket;

#[cfg(unix)]
#[path = "vmsocket.linux.rs"]
mod vmsocket;

use async_std::net::{Shutdown, TcpStream};
use futures_util::future::try_join;

#[cfg(windows)]
use uuid::Uuid;

#[cfg(unix)]
use async_std::os::unix::net::{UnixListener, UnixStream};

use vmsocket::VmSocket;

trait Stream: async_std::io::Read + async_std::io::Write + Clone + Unpin {
    fn shutdown(&self, how: std::net::Shutdown) -> std::io::Result<()>;
}

impl Stream for &TcpStream {
    fn shutdown(&self, how: std::net::Shutdown) -> std::io::Result<()> {
        TcpStream::shutdown(self, how)
    }
}

#[cfg(unix)]
impl Stream for &UnixStream {
    fn shutdown(&self, how: std::net::Shutdown) -> std::io::Result<()> {
        UnixStream::shutdown(self, how)
    }
}

async fn connect_stream<C: Stream, S: Stream>(client: C, server: S) -> std::io::Result<()> {
    let c2s = async {
        async_std::io::copy(&mut client.clone(), &mut server.clone()).await?;
        server.shutdown(Shutdown::Write)
    };

    let s2c = async {
        async_std::io::copy(&mut server.clone(), &mut client.clone()).await?;
        client.shutdown(Shutdown::Write)
    };

    try_join(c2s, s2c).await?;
    Ok(())
}

#[cfg(windows)]
async fn task(vmid: Uuid) -> std::io::Result<()> {
    let listener = VmSocket::bind(vmid, 6000).await?;

    loop {
        let client = listener.accept().await?;

        async_std::task::spawn(async move {
            let result = async {
                let server = TcpStream::connect("localhost:6000").await?;
                server.set_nodelay(true)?;
                connect_stream(&client, &server).await
            }
            .await;
            if let Err(err) = result {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

#[cfg(windows)]
fn main() {
    unsafe { winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) };

    let vmid_arg = std::env::args().nth(1);

    if let Some("--daemon") = vmid_arg.as_deref() {
        let mut prev_vmid = None;
        let mut future: Option<async_std::task::JoinHandle<()>> = None;
        loop {
            let vmid = vmcompute::get_wsl_vmid().unwrap();
            if vmid != prev_vmid {
                if let Some(future) = future.take() {
                    async_std::task::block_on(future.cancel());
                }
                prev_vmid = vmid;
                if let Some(vmid) = vmid {
                    future = Some(async_std::task::spawn(async move {
                        // Three chances, to avoid a race between get_wsl_vmid and spawn.
                        for _ in 0..3 {
                            if let Err(err) = task(vmid).await {
                                eprintln!("Failed to listen: {}", err);
                            }
                            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
                        }
                        std::process::exit(1);
                    }));
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    } else {
        let vmid = match vmid_arg {
            Some(str) => str.parse().expect("VMID is not valid UUID"),
            None => vmcompute::get_wsl_vmid().unwrap().expect("WSL is not running"),
        };

        async_std::task::block_on(async {
            if let Err(err) = task(vmid).await {
                eprintln!("Failed to listen: {}", err);
                return;
            }
        });
    }
}

#[cfg(unix)]
async fn task() -> std::io::Result<()> {
    let listener = UnixListener::bind("/tmp/.X11-unix/X0").await?;

    loop {
        let (client, _) = listener.accept().await?;

        async_std::task::spawn(async move {
            let result = async {
                let server = VmSocket::connect(6000).await?;
                connect_stream(&client, &server).await
            }
            .await;
            if let Err(err) = result {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

#[cfg(unix)]
fn main() {
    // Remove existing socket
    let _ = std::fs::create_dir_all("/tmp/.X11-unix");
    let _ = std::fs::remove_file("/tmp/.X11-unix/X0");

    async_std::task::block_on(async {
        if let Err(err) = task().await {
            eprintln!("Failed to listen: {}", err);
            return;
        }
    });
}
