#[allow(unused_imports)]
use async_std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use futures_util::future::try_join;

#[cfg(windows)]
mod windows {
    
    use async_io::Async;
    use async_std::net::TcpStream;

    pub mod sync {
        use std::net::TcpStream;
        use std::os::windows::io::{AsRawSocket, FromRawSocket, RawSocket};
        use uuid::Uuid;
        use once_cell::sync::Lazy;

        use winapi::shared::ws2def::SOCK_STREAM;
        use winapi::shared::ws2def::*;
        use winapi::um::winsock2::*;

        #[repr(C)]
        #[derive(Clone, Copy)]
        #[allow(non_snake_case)]
        struct SOCKADDR_HV {
            pub Family: ADDRESS_FAMILY,
            pub Reserved: winapi::shared::minwindef::USHORT,
            pub VmId: winapi::shared::guiddef::GUID,
            pub ServiceId: winapi::shared::guiddef::GUID,
        }

        const HV_PROTOCOL_RAW: winapi::ctypes::c_int = 1;

        static VMID: Lazy<Uuid> = Lazy::new(|| {
            let vmid_str = std::env::args().nth(1).expect("VMID not supplied");
            let vmid: Uuid = vmid_str.parse().expect("VMID is not valid UUID");
            vmid
        });

        fn last_error() -> std::io::Error {
            std::io::Error::from_raw_os_error(unsafe { winapi::um::winsock2::WSAGetLastError() })
        }

        pub struct VmSocket(SOCKET);

        impl AsRawSocket for VmSocket {
            fn as_raw_socket(&self) -> RawSocket {
                self.0 as _
            }
        }

        impl Drop for VmSocket {
            fn drop(&mut self) {
                unsafe {
                    closesocket(self.0);
                }
            }
        }

        impl VmSocket {
            pub fn bind(port: u32) -> std::io::Result<VmSocket> {
                unsafe {
                    let mut local_addr: SOCKADDR_HV = std::mem::zeroed();
                    local_addr.Family = AF_HYPERV as _;
                    // Set GUID to "00000000-facb-11e6-bd58-64006a7986d3" with Data1 set as port desired.
                    let service_id: Uuid = "00000000-facb-11e6-bd58-64006a7986d3".parse().unwrap();
                    let parts = service_id.as_fields();
                    local_addr.ServiceId.Data1 = port as _;
                    local_addr.ServiceId.Data2 = parts.1;
                    local_addr.ServiceId.Data3 = parts.2;
                    local_addr.ServiceId.Data4 = *parts.3;
                    let parts = VMID.as_fields();
                    local_addr.VmId.Data1 = parts.0;
                    local_addr.VmId.Data2 = parts.1;
                    local_addr.VmId.Data3 = parts.2;
                    local_addr.VmId.Data4 = *parts.3;

                    let fd = socket(AF_HYPERV, SOCK_STREAM, HV_PROTOCOL_RAW);
                    if fd == INVALID_SOCKET {
                        return Err(last_error());
                    }

                    let result = bind(
                        fd,
                        &local_addr as *const _ as *const SOCKADDR,
                        std::mem::size_of::<SOCKADDR_HV>() as _,
                    );
                    if result < 0 {
                        let err = last_error();
                        closesocket(fd);
                        return Err(err);
                    }

                    let result = listen(fd, SOMAXCONN);
                    if result < 0 {
                        let err = last_error();
                        closesocket(fd);
                        return Err(err);
                    }

                    Ok(VmSocket(fd))
                }
            }

            pub fn accept(&self) -> std::io::Result<TcpStream> {
                let fd = unsafe { accept(self.0, std::ptr::null_mut(), std::ptr::null_mut()) };
                if fd == INVALID_SOCKET {
                    return Err(last_error());
                }
                Ok(unsafe { TcpStream::from_raw_socket(fd as _) })
            }
        }
    }

    pub struct VmSocket(Async<sync::VmSocket>);

    impl VmSocket {
        pub async fn bind(port: u32) -> std::io::Result<Self> {
            Ok(Self(Async::new(sync::VmSocket::bind(port)?)?))
        }

        pub async fn accept(&self) -> std::io::Result<TcpStream> {
            let stream = self.0.read_with(|io| io.accept()).await?;
            Ok(stream.into())
        }
    }
}

#[cfg(windows)]
use crate::windows::*;

#[cfg(unix)]
mod linux {
    use async_std::net::TcpStream;

    pub mod sync {
        use std::net::TcpStream;
        use std::os::unix::io::FromRawFd;

        pub struct VmSocket;

        impl VmSocket {
            pub fn connect(port: u32) -> std::io::Result<TcpStream> {
                unsafe {
                    let mut local_addr: libc::sockaddr_vm = std::mem::zeroed();
                    local_addr.svm_family = libc::AF_VSOCK as _;
                    local_addr.svm_port = libc::VMADDR_PORT_ANY as _;
                    local_addr.svm_cid = libc::VMADDR_CID_ANY as _;

                    let mut rem_addr: libc::sockaddr_vm = std::mem::zeroed();
                    rem_addr.svm_family = libc::AF_VSOCK as _;
                    rem_addr.svm_port = port as _;
                    rem_addr.svm_cid = libc::VMADDR_CID_HOST as _;

                    let fd = libc::socket(libc::AF_VSOCK, libc::SOCK_STREAM, 0);
                    if fd < 0 {
                        return Err(std::io::Error::last_os_error());
                    }

                    let result = libc::bind(
                        fd,
                        &local_addr as *const _ as *mut libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_vm>() as _,
                    );
                    if result < 0 {
                        let err = std::io::Error::last_os_error();
                        libc::close(fd);
                        return Err(err);
                    }

                    let result = libc::connect(
                        fd,
                        &rem_addr as *const _ as *mut libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_vm>() as _,
                    );
                    if result < 0 {
                        let err = std::io::Error::last_os_error();
                        libc::close(fd);
                        return Err(err);
                    }

                    Ok(TcpStream::from_raw_fd(fd))
                }
            }
        }
    }

    pub struct VmSocket;

    impl VmSocket {
        pub async fn connect(port: u32) -> std::io::Result<TcpStream> {
            Ok(sync::VmSocket::connect(port)?.into())
        }
    }
}

#[cfg(unix)]
use crate::linux::*;

async fn connect_stream(client: TcpStream, server: TcpStream) -> std::io::Result<()> {
    let c2s = async {
        async_std::io::copy(&mut &client, &mut &server).await?;
        server.shutdown(Shutdown::Write)
    };

    let s2c = async {
        async_std::io::copy(&mut &server, &mut &client).await?;
        client.shutdown(Shutdown::Write)
    };

    try_join(c2s, s2c).await?;
    Ok(())
}

async fn task() -> std::io::Result<()> {
    // This will trigger WinSock2 initialisation
    #[cfg(windows)]
    let remote_addr = "localhost:6000"
        .to_socket_addrs()
        .await
        .unwrap()
        .next()
        .unwrap();

    #[cfg(unix)]
    let listener = TcpListener::bind("127.0.0.1:6001").await?;
    #[cfg(windows)]
    let listener = VmSocket::bind(6000).await?;

    loop {
        #[cfg(unix)]
        let client = {
            let (stream, _) = listener.accept().await?;
            let _ = stream.set_nodelay(true);
            stream
        };
        #[cfg(windows)]
        let client = listener.accept().await?;

        async_std::task::spawn(async move {
            let result = async {
                #[cfg(unix)]
                let server = VmSocket::connect(6000).await?;
                #[cfg(windows)]
                let server = {
                    let stream = TcpStream::connect(remote_addr).await?;
                    stream.set_nodelay(true)?;
                    stream
                };
                connect_stream(client, server).await
            }
            .await;
            if let Err(err) = result {
                eprintln!("Failed to transfer: {}", err);
            }
        });
    }
}

fn main() {
    async_std::task::block_on(async {
        if let Err(err) = task().await {
            eprintln!("Failed to listen: {}", err);
            return;
        }
    });
}
