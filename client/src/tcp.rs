use super::config::TcpForwardConfig;
use super::util::{connect_stream, either};
use super::vmsocket::VmSocket;
use super::CONFIG;

use log::{info, warn};
use std::io::{Error, ErrorKind, Result as IoResult};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

fn get_origin_dst(stream: &TcpStream) -> IoResult<SocketAddr> {
    use std::mem;
    use std::os::unix::io::AsRawFd;

    unsafe {
        let mut storage: libc::sockaddr_in6 = mem::zeroed();
        let mut socklen = mem::size_of_val(&storage) as libc::socklen_t;

        let (sol, so) = match stream.local_addr()? {
            SocketAddr::V4(_) => (libc::SOL_IP, libc::SO_ORIGINAL_DST),
            SocketAddr::V6(_) => (libc::SOL_IPV6, libc::IP6T_SO_ORIGINAL_DST),
        };

        let ret = libc::getsockopt(
            stream.as_raw_fd(),
            sol,
            so,
            &mut storage as *mut _ as *mut libc::c_void,
            &mut socklen,
        );
        if ret < 0 {
            return Err(Error::last_os_error());
        }

        let addr = match storage.sin6_family as libc::c_int {
            libc::AF_INET => SocketAddr::V4(mem::transmute(
                *(&storage as *const _ as *const libc::sockaddr_in),
            )),
            libc::AF_INET6 => SocketAddr::V6(mem::transmute(storage)),
            _ => return Err(Error::new(ErrorKind::InvalidData, "unknown address family")),
        };

        Ok(addr)
    }
}

async fn handle_stream(
    config: &'static TcpForwardConfig,
    mut stream: TcpStream,
    peer: SocketAddr,
) -> std::io::Result<()> {
    let local_addr = get_origin_dst(&stream)?;
    let port = local_addr.port();

    info!("{} connected to {}", peer, port);

    if port == config.service_port {
        // Disallow direct connection to this port.
        warn!("connection to service port {} is disallowed", port);
        return Ok(());
    }

    stream.set_nodelay(true)?;

    let mut server = VmSocket::connect(CONFIG.service_port).await?;
    server.write_all(b"tcp\0").await?;
    server.write_u16(port).await?;

    let (client_r, client_w) = stream.split();
    let (server_r, server_w) = server.split();
    let a = connect_stream(client_r, server_w);
    let b = connect_stream(server_r, client_w);
    either(a, b).await
}

pub async fn execute_iptables(config: &'static TcpForwardConfig, cmd: &str) -> std::io::Result<Result<(), ()>> {
    let mut p = tokio::process::Command::new("sh");
    p.arg("-c");
    p.arg(format!("{} -t nat {}", config.iptables_cmd, cmd));
    let mut child = p.spawn()?;
    let exit = child.wait().await?;
    Ok(if exit.success() { Ok(()) } else { Err(()) })
}

pub async fn tcp_forward(config: &'static TcpForwardConfig) -> std::io::Result<()> {
    let listener = TcpListener::bind(("localhost", config.service_port)).await?;

    let _ = execute_iptables(config, "-N wsld").await?;
    execute_iptables(config, "-F wsld").await?.unwrap();
    let _ = execute_iptables(config, "-D OUTPUT -j wsld").await?;
    execute_iptables(config, "-I OUTPUT -j wsld").await?.unwrap();

    for &port in config.ports.iter() {
        execute_iptables(config, &format!("-A wsld -p tcp --dport {} -j REDIRECT --to-port {}", port, config.service_port)).await?.unwrap();
    }
    execute_iptables(config, "-A wsld -j RETURN").await?.unwrap();

    loop {
        let (stream, peer) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = handle_stream(config, stream, peer).await {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}
