use std::convert::TryInto;
use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn handle_time(mut stream: TcpStream) -> std::io::Result<()> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    stream
        .write_u64(
            time.as_micros()
                .try_into()
                .expect("timestamp should fit in u64"),
        )
        .await
}
