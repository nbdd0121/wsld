// Hide console window
#![windows_subsystem = "windows"]

mod time;
mod vmcompute;
mod vmsocket;

use std::io::{Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::pin;
use uuid::Uuid;

use vmsocket::VmSocket;

// The Hyper-V socket used for service.
const SERVICE_PORT: u32 = 6000;

async fn connect_stream<R: AsyncRead, W: AsyncWrite>(r: R, w: W) -> std::io::Result<()> {
    pin!(r);
    pin!(w);
    let mut buf = vec![0u8; 4096];
    loop {
        let size = r.read(&mut buf).await?;
        if size == 0 {
            break;
        }
        w.write_all(&buf[0..size]).await?;
    }
    w.shutdown().await
}

async fn handle_x11(stream: TcpStream) -> std::io::Result<()> {
    let (client_r, client_w) = stream.into_split();

    let server = TcpStream::connect("localhost:6000").await?;
    server.set_nodelay(true)?;
    let (server_r, server_w) = server.into_split();
    let a = tokio::task::spawn(connect_stream(client_r, server_w));
    let b = tokio::task::spawn(connect_stream(server_r, client_w));
    a.await.unwrap()?;
    b.await.unwrap()
}

async fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    // Read the function code at the start of the stream for demultiplexing
    let func = {
        let mut buf = [0; 4];
        stream.read_exact(&mut buf).await?;
        buf
    };

    match &func {
        b"x11\0" => handle_x11(stream).await,
        b"time" => time::handle_time(stream).await,
        b"noop" => Ok(()),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("unknown function {:?}", func),
        )),
    }
}

async fn task(vmid: Uuid) -> std::io::Result<()> {
    let listener = VmSocket::bind(vmid, SERVICE_PORT).await?;

    loop {
        let stream = listener.accept().await?;

        tokio::task::spawn(async move {
            let result = handle_stream(stream).await;
            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        });
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    unsafe { winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) };

    let vmid_arg = std::env::args().nth(1);

    if let Some("--daemon") = vmid_arg.as_deref() {
        let mut prev_vmid = None;
        let mut future: Option<tokio::task::JoinHandle<()>> = None;
        loop {
            let vmid = tokio::task::spawn_blocking(|| vmcompute::get_wsl_vmid().unwrap())
                .await
                .unwrap();
            if vmid != prev_vmid {
                if let Some(future) = future.take() {
                    future.abort();
                }
                prev_vmid = vmid;
                if let Some(vmid) = vmid {
                    future = Some(tokio::task::spawn(async move {
                        // Three chances, to avoid a race between get_wsl_vmid and spawn.
                        for _ in 0..3 {
                            if let Err(err) = task(vmid).await {
                                eprintln!("Failed to listen: {}", err);
                            }
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                        std::process::exit(1);
                    }));
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    } else {
        let vmid = match vmid_arg {
            Some(str) => str.parse().expect("VMID is not valid UUID"),
            None => vmcompute::get_wsl_vmid()
                .unwrap()
                .expect("WSL is not running"),
        };

        if let Err(err) = task(vmid).await {
            eprintln!("Failed to listen: {}", err);
            return;
        }
    }
}
