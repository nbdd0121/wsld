// Hide console window
#![windows_subsystem = "windows"]

mod config;
mod ssh_agent;
mod tcp;
mod time;
mod util;
mod vmcompute;
mod vmsocket;
mod x11;

use once_cell::sync::Lazy;
use std::io::{Error, ErrorKind};
use structopt::StructOpt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use uuid::Uuid;

use config::Config;
use vmsocket::VmSocket;

static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_args());

async fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    // Read the function code at the start of the stream for demultiplexing
    let func = {
        let mut buf = [0; 4];
        stream.read_exact(&mut buf).await?;
        buf
    };

    match &func {
        b"x11\0" => x11::handle_x11(stream).await,
        b"time" => time::handle_time(stream).await,
        b"tcp\0" => tcp::handle_tcp(stream).await,
        b"ssha" => ssh_agent::handle_ssh_agent(stream).await,
        b"noop" => Ok(()),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("unknown function {:?}", func),
        )),
    }
}

async fn task(vmid: Uuid) -> std::io::Result<()> {
    let listener = VmSocket::bind(vmid, CONFIG.service_port).await?;

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

    if CONFIG.daemon {
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
        let vmid = match CONFIG.vmid {
            Some(str) => str,
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
