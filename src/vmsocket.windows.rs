use async_io::Async;
use std::convert::TryInto;
use tokio::net::TcpStream;

pub mod sync {
    use once_cell::sync::Lazy;
    use std::net::TcpStream;
    use std::net::ToSocketAddrs;
    use std::os::windows::io::{AsRawSocket, FromRawSocket, RawSocket};
    use uuid::Uuid;

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

    fn init() {
        static GUARD: Lazy<()> = Lazy::new(|| {
            // This will trigger WinSock2 initialisation
            "localhost:6000".to_socket_addrs().unwrap().next().unwrap();
        });
        *GUARD
    }

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
        pub fn bind(vmid: Uuid, port: u32) -> std::io::Result<VmSocket> {
            init();
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
                let parts = vmid.as_fields();
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
    pub async fn bind(vmid: uuid::Uuid, port: u32) -> std::io::Result<Self> {
        Ok(Self(Async::new(sync::VmSocket::bind(vmid, port)?)?))
    }

    pub async fn accept(&self) -> std::io::Result<TcpStream> {
        let stream = self.0.read_with(|io| io.accept()).await?;
        stream.set_nonblocking(true)?;
        stream.try_into()
    }
}
