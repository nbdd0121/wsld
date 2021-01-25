use super::config::X11Config;
use super::vmsocket::VmSocket;
use super::x11socket::X11Lock;
use super::CONFIG;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;

async fn connect_stream<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut r: R,
    mut w: W,
) -> std::io::Result<()> {
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

async fn handle_stream(stream: UnixStream) -> std::io::Result<()> {
    let mut server = VmSocket::connect(CONFIG.service_port).await?;
    server.write_all(b"x11\0").await?;

    let (client_r, client_w) = stream.into_split();
    let (server_r, server_w) = server.into_split();
    let a = tokio::task::spawn(connect_stream(client_r, server_w));
    let b = tokio::task::spawn(connect_stream(server_r, client_w));
    a.await.unwrap()?;
    b.await.unwrap()
}

pub async fn x11_forward(config: &'static X11Config) -> std::io::Result<()> {
    let lock = X11Lock::acquire(config.display, config.force)?;
    let listener = lock.bind()?;

    loop {
        let stream = listener.accept().await?.0;

        tokio::task::spawn(async move {
            if let Err(err) = handle_stream(stream).await {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}
