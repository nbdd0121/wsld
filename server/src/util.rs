use std::future::Future;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn either<T, A: Future<Output = T>, B: Future<Output = T>>(a: A, b: B) -> T {
    tokio::select! {
        a = a => a,
        b = b => b,
    }
}

pub async fn connect_stream<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
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
