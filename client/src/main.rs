mod time;
mod vmsocket;
mod x11socket;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::pin;

use vmsocket::VmSocket;

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

async fn task() -> std::io::Result<()> {
    let lock = x11socket::X11Lock::acquire(0)?;
    let listener = lock.bind()?;

    loop {
        let (client_r, client_w) = listener.accept().await?.0.into_split();

        tokio::task::spawn(async move {
            let result = async {
                let (server_r, server_w) = VmSocket::connect(6000).await?.into_split();
                let a = tokio::task::spawn(connect_stream(client_r, server_w));
                let b = tokio::task::spawn(connect_stream(server_r, client_w));
                a.await.unwrap()?;
                b.await.unwrap()
            }
            .await;
            if let Err(err) = result {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = task().await {
        eprintln!("Failed to listen: {}", err);
        return;
    }
}
