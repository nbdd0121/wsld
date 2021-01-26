use super::util::{connect_stream, either};

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub async fn handle_tcp(mut stream: TcpStream) -> std::io::Result<()> {
    let port = stream.read_u16().await?;
    let (client_r, client_w) = stream.split();

    let mut server = TcpStream::connect(("127.0.0.1", port)).await?;
    server.set_nodelay(true)?;
    let (server_r, server_w) = server.split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}
