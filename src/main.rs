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

#[cfg(unix)]
use async_std::os::unix::net::{UnixListener, UnixStream};

use vmsocket::VmSocket;

#[cfg(unix)]
use crate::linux::*;

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

async fn task() -> std::io::Result<()> {
    #[cfg(unix)]
    let listener = UnixListener::bind("/tmp/.X11-unix/X0").await?;
    #[cfg(windows)]
    let listener = VmSocket::bind(6000).await?;

    loop {
        #[cfg(unix)]
        let (client, _) = listener.accept().await?;
        #[cfg(windows)]
        let client = listener.accept().await?;

        async_std::task::spawn(async move {
            let result = async {
                #[cfg(unix)]
                let server = VmSocket::connect(6000).await?;
                #[cfg(windows)]
                let server = {
                    let stream = TcpStream::connect("localhost:6000").await?;
                    stream.set_nodelay(true)?;
                    stream
                };
                connect_stream(&client, &server).await
            }
            .await;
            if let Err(err) = result {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

fn main() {
    #[cfg(windows)]
    {
        unsafe { winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) };
    }

    #[cfg(unix)]
    {
        let _ = std::fs::create_dir_all("/tmp/.X11-unix");
        let _ = std::fs::remove_file("/tmp/.X11-unix/X0");
    }

    async_std::task::block_on(async {
        if let Err(err) = task().await {
            eprintln!("Failed to listen: {}", err);
            return;
        }
    });
}
