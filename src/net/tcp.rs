use core::net::SocketAddr;

use crate::sys::addr::SocketAddrExt;
use crate::io::{Read, Write, Result};

use super::sys::{RawSocket, Socket, SOCK_STREAM};
use super::{AsRawSocket, ToSocketAddrs, for_each_addr};

/// A tcp socket which have not yet been bound
///
/// You may use it for better control over socket options - however, it doesn't support [`ToSocketAddrs`]. [`TcpListener::bind`] or [`TcpStream::connect`] is a more convenient alternative
pub struct TcpSocket {
    socket: Socket,
}

impl TcpSocket {
    /// Creates a new tcp socket. `addr` is only used to determine address family
    pub fn new(addr: SocketAddr) -> Result<TcpSocket> {
        let socket = Socket::new(addr.address_family(), SOCK_STREAM)?;
        Ok(TcpSocket { socket })
    }

    /// Creates a new non-blocking tcp socket. `addr` is only used to determine address family
    pub fn new_nonblock(addr: SocketAddr) -> Result<TcpSocket> {
        let socket = Socket::new_nonblock(addr.address_family(), SOCK_STREAM)?;
        Ok(TcpSocket { socket })
    }

    /// Binds the socket to a local address
    pub fn bind(&self, addr: SocketAddr) -> Result<()> {
        self.socket.bind(addr)
    }

    /// Begins listening, and returns a [`TcpListener`]
    pub fn listen(self) -> Result<TcpListener> {
        self.socket.listen()?;
        let socket = self.socket;
        Ok(TcpListener { socket })
    }

    // TODO: connect
}

impl AsRawSocket for TcpSocket {
    fn as_raw_socket(&self) -> RawSocket {
        self.socket.as_raw_socket()
    }

    fn into_raw_socket(self) -> RawSocket {
        self.socket.into_raw_socket()
    }

    fn from_raw_socket(socket: RawSocket) -> TcpSocket {
        TcpSocket { socket: Socket::from_raw_socket(socket) }
    }
}

/// A TCP socket server, listening for connections
pub struct TcpListener {
    socket: Socket,
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified address
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<TcpListener> {
        for_each_addr(addr, |addr| {
            let sock = TcpSocket::new(addr)?;
            sock.bind(addr)?;
            sock.listen()
        })
    }

    /// Accept a new incoming connection from this listener
    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        self.accept_ex(false)
    }

    /// Accept a new incoming connection from this listener, optionally in non-blocking mode
    ///
    /// `nonblock` controls if the newly accepted socket should be nonblocking. Leave it as `false` unless you are using an async runtime
    pub fn accept_ex(&self, nonblock: bool) -> Result<(TcpStream, SocketAddr)> {
        let (socket, addr) = self.socket.accept(nonblock)?;
        Ok((TcpStream { socket }, addr))
    }
}

impl AsRawSocket for TcpListener {
    fn as_raw_socket(&self) -> RawSocket {
        self.socket.as_raw_socket()
    }

    fn into_raw_socket(self) -> RawSocket {
        self.socket.into_raw_socket()
    }

    fn from_raw_socket(socket: RawSocket) -> TcpListener {
        TcpListener { socket: Socket::from_raw_socket(socket) }
    }
}

/// A TCP stream between a local and a remote socket
// TODO: peek, shutdown, connect
pub struct TcpStream {
    socket: Socket,
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.socket.read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.socket.write(buf)
    }
}

impl AsRawSocket for TcpStream {
    fn as_raw_socket(&self) -> RawSocket {
        self.socket.as_raw_socket()
    }

    fn into_raw_socket(self) -> RawSocket {
        self.socket.into_raw_socket()
    }

    fn from_raw_socket(socket: RawSocket) -> TcpStream {
        TcpStream { socket: Socket::from_raw_socket(socket) }
    }
}
