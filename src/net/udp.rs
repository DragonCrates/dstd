use core::net::SocketAddr;

use crate::sys::addr::SocketAddrExt;
use crate::io::{self, Error};

use super::sys::{RawSocket, Socket, SOCK_DGRAM};
use super::{AsRawSocket, ToSocketAddrs, for_each_addr};

/// A UDP socket.
///
/// UDP is a connectionless and message-oriented protocol. This struct allows
/// sending and receiving datagrams to and from any address.
pub struct UdpSocket {
    socket: Socket
}

impl UdpSocket {
    /// Creates a new UDP socket. `addr` is only used to determine address family.
    pub fn new(addr: SocketAddr) -> io::Result<UdpSocket> {
        let socket = Socket::new(addr.address_family(), SOCK_DGRAM)?;
        Ok(UdpSocket { socket })
    }

    /// Creates a new non-blocking UDP socket. `addr` is only used to determine address family.
    pub fn new_nonblock(addr: SocketAddr) -> io::Result<UdpSocket> {
        let socket = Socket::new_nonblock(addr.address_family(), SOCK_DGRAM)?;
        Ok(UdpSocket { socket })
    }

    /// Binds the socket to the specified address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        for_each_addr(addr, |addr| {
            let sock = UdpSocket::new(addr)?;
            sock.socket.bind(addr)?;
            Ok(sock)
        })
    }

    /// Connects this UDP socket to a remote address.
    ///
    /// After a successful call to `connect`, `send` and `recv` can be used
    /// instead of `send_to` and `recv_from`.
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        for_each_addr(addr, |addr| {
            self.socket.connect(addr)?;
            Ok(())
        })
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    ///
    /// Only the first address from the [`ToSocketAddrs`] iterator is used. If their address family doesn't match (ipv6 address on ipv4 socket), you will get an error - so please specify an address explicitly
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        let addr = match addr.to_socket_addrs()?.next() {
            Some(a) => a,
            None => return Err(Error::new_no_addresses()),
        };
        self.socket.send_to(buf, addr)
    }

    /// Receives data from the socket from the remote address to which it is connected.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }

    /// Receives data from the socket. On success, returns the number of bytes read
    /// and the address from which the data came.
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    // TODO: peek, peek_from
}

impl AsRawSocket for UdpSocket {
    fn as_raw_socket(&self) -> RawSocket {
        self.socket.as_raw_socket()
    }

    fn into_raw_socket(self) -> RawSocket {
        self.socket.into_raw_socket()
    }

    fn from_raw_socket(socket: RawSocket) -> UdpSocket {
        UdpSocket { socket: Socket::from_raw_socket(socket) }
    }
}
