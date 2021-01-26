use super::util::{connect_stream, either};
use super::CONFIG;

use tokio::net::TcpStream;

pub async fn handle_x11(stream: TcpStream) -> std::io::Result<()> {
    let (client_r, client_w) = stream.into_split();

    let server = TcpStream::connect(&CONFIG.x11.display).await?;
    server.set_nodelay(true)?;
    let (server_r, server_w) = server.into_split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}
