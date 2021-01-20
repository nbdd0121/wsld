use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

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

pub async fn handle_x11(stream: TcpStream) -> std::io::Result<()> {
    let (client_r, client_w) = stream.into_split();

    let server = TcpStream::connect("localhost:6000").await?;
    server.set_nodelay(true)?;
    let (server_r, server_w) = server.into_split();
    let a = tokio::task::spawn(connect_stream(client_r, server_w));
    let b = tokio::task::spawn(connect_stream(server_r, client_w));
    a.await.unwrap()?;
    b.await.unwrap()
}
