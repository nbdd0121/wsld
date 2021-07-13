use super::util::{connect_stream, either};

use tokio::net::windows::named_pipe;
use tokio::net::TcpStream;

pub async fn handle_ssh_agent(mut stream: TcpStream) -> std::io::Result<()> {
    let (client_r, client_w) = stream.split();

    let server = named_pipe::ClientOptions::new().open(r"\\.\pipe\openssh-ssh-agent")?;
    let (server_r, server_w) = tokio::io::split(server);
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}
