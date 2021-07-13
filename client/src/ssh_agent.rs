use super::config::SshAgentConfig;
use super::util::{connect_stream, either};
use super::vmsocket::VmSocket;
use super::CONFIG;

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

async fn handle_stream(mut stream: UnixStream) -> std::io::Result<()> {
    let mut server = VmSocket::connect(CONFIG.service_port).await?;
    server.write_all(b"ssha").await?;

    let (client_r, client_w) = stream.split();
    let (server_r, server_w) = server.split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}

pub async fn ssh_agent_forward(config: &'static SshAgentConfig) -> std::io::Result<()> {
    // Remove existing socket
    let _ = std::fs::create_dir_all(Path::new(&config.ssh_auth_sock).parent().unwrap());
    let _ = std::fs::remove_file(&config.ssh_auth_sock);

    let listener = UnixListener::bind(&config.ssh_auth_sock)?;
    let _ = std::fs::set_permissions(&config.ssh_auth_sock, Permissions::from_mode(0o600));

    loop {
        let stream = listener.accept().await?.0;

        tokio::task::spawn(async move {
            if let Err(err) = handle_stream(stream).await {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}
