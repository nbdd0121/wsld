use std::convert::TryInto;
use tokio::net::TcpStream;

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
        let stream = sync::VmSocket::connect(port)?;
        stream.set_nonblocking(true)?;
        stream.try_into()
    }
}
