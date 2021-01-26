use super::config::X11Config;
use super::util::{connect_stream, either};
use super::vmsocket::VmSocket;
use super::x11socket::X11Lock;
use super::CONFIG;

use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

async fn handle_stream(stream: UnixStream) -> std::io::Result<()> {
    let mut server = VmSocket::connect(CONFIG.service_port).await?;
    server.write_all(b"x11\0").await?;

    let (client_r, client_w) = stream.into_split();
    let (server_r, server_w) = server.into_split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
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
