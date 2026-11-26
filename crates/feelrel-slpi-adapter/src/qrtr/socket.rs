use std::io;
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::ptr;

use tokio::io::{Interest, unix::AsyncFd};

use super::address::{AF_QIPCRTR, QrtrAddress};

const ADDRESS_SIZE: libc::socklen_t = size_of::<QrtrAddress>() as libc::socklen_t;

pub(crate) struct QrtrSocket(AsyncFd<OwnedFd>);

impl QrtrSocket {
    pub fn open() -> io::Result<Self> {
        // SAFETY: The constants select a datagram socket with no pointer parameters.
        let fd = unsafe {
            libc::socket(
                AF_QIPCRTR,
                libc::SOCK_DGRAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
                0,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: A successful socket call returns a new descriptor owned by this function.
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: OwnedFd exclusively holds the socket and keeps its descriptor stable.
        let socket = unsafe { AsyncFd::register(fd) }.map_err(|error| error.into_parts().1)?;
        Ok(Self(socket))
    }

    pub fn local_address(&self) -> io::Result<QrtrAddress> {
        let mut address = QrtrAddress::new(0, 0);
        let mut length = ADDRESS_SIZE;
        // SAFETY: Both outputs are writable and the live fd remains borrowed through the call.
        let result = unsafe {
            libc::getsockname(
                self.0.as_raw_fd(),
                ptr::from_mut(&mut address).cast(),
                &mut length,
            )
        };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        validate_address(address, length)
    }

    pub async fn send_to(&self, destination: QrtrAddress, packet: &[u8]) -> io::Result<()> {
        self.0
            .async_io(Interest::WRITABLE, |fd| {
                // SAFETY: The socket, packet, and sockaddr outlive this nonblocking call.
                let sent = unsafe {
                    libc::sendto(
                        fd.as_raw_fd(),
                        packet.as_ptr().cast(),
                        packet.len(),
                        0,
                        ptr::from_ref(&destination).cast(),
                        ADDRESS_SIZE,
                    )
                };
                let sent = byte_count(sent)?;
                if sent != packet.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "incomplete QRTR datagram",
                    ));
                }
                Ok(())
            })
            .await
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> io::Result<(QrtrAddress, usize)> {
        self.0
            .async_io(Interest::READABLE, |fd| {
                let mut address = QrtrAddress::new(0, 0);
                let mut vector = libc::iovec {
                    iov_base: buffer.as_mut_ptr().cast(),
                    iov_len: buffer.len(),
                };
                // SAFETY: The zeroed optional fields of msghdr are null pointers or zero lengths.
                let mut header: libc::msghdr = unsafe { std::mem::zeroed() };
                header.msg_name = ptr::from_mut(&mut address).cast();
                header.msg_namelen = ADDRESS_SIZE;
                header.msg_iov = &mut vector;
                header.msg_iovlen = 1;
                // SAFETY: The writable address and buffer outlive this nonblocking call.
                let received =
                    byte_count(unsafe { libc::recvmsg(fd.as_raw_fd(), &mut header, 0) })?;
                if header.msg_flags & libc::MSG_TRUNC != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "truncated QRTR datagram",
                    ));
                }
                Ok((validate_address(address, header.msg_namelen)?, received))
            })
            .await
    }
}

fn validate_address(address: QrtrAddress, length: libc::socklen_t) -> io::Result<QrtrAddress> {
    if length != ADDRESS_SIZE || libc::c_int::from(address.family) != AF_QIPCRTR {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid QRTR socket address",
        ));
    }
    Ok(address)
}

fn byte_count(result: libc::ssize_t) -> io::Result<usize> {
    usize::try_from(result).map_err(|_| io::Error::last_os_error())
}
