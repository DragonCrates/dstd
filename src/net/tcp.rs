use core::mem::{self, ManuallyDrop};
use core::net::SocketAddr;
use core::ffi::c_int;

use crate::io::{Read, Write, Result};

use super::sys::*;
use super::AsSocket;

// TODO:
// - set nonblocking
// - getaddrinfo

pub struct TcpSocket {
    handle: Socket,
}

impl TcpSocket {
    pub fn new(addr: SocketAddr) -> Result<TcpSocket> {
        let handle = new_cloexec(addr.address_family() as c_int, SOCK_STREAM);
        if handle == INVALID_SOCKET { return Err(socketerror()); }
        // TODO: SO_REUSEPORT
        Ok(TcpSocket { handle })
    }

    pub fn new_nonblock(addr: SocketAddr) -> Result<TcpSocket> {
        let handle = new_nonblock(addr.address_family() as c_int, SOCK_STREAM);
        if handle == INVALID_SOCKET { return Err(socketerror()); }
        // TODO: SO_REUSEPORT
        Ok(TcpSocket { handle })
    }

    // TODO: bind, listen, connect
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        unsafe { closesocket(self.handle); }
    }
}

impl AsSocket for TcpSocket {
    fn as_socket(&self) -> Socket {
        self.handle
    }

    fn into_socket(self) -> Socket {
        let me = ManuallyDrop::new(self);
        me.handle
    }

    fn from_socket(socket: Socket) -> TcpSocket {
        TcpSocket { handle: socket }
    }
}

/// A TCP socket server, listening for connections
pub struct TcpListener {
    handle: Socket,
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified address
    pub fn bind(addr: SocketAddr) -> Result<TcpListener> {
        init();

        unsafe {
            let handle = TcpSocket::new(addr)?.into_socket();
            let listener = TcpListener { handle };

            let sockaddr = addr.to_sockaddr();
            let ret = bind(handle, &sockaddr, mem::size_of::<sockaddr>() as socklen_t);
            if ret == -1 { return Err(socketerror()); }
            let ret = listen(handle, SOMAXCONN);
            if ret == -1 { return Err(socketerror()); }
            Ok(listener)
        }
    }

    /// Accept a new incoming connection from this listener
    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let mut addr = sockaddr::default();
        let mut addrlen = mem::size_of::<sockaddr>() as socklen_t;
        let handle = unsafe { accept(self.handle, &mut addr, &mut addrlen) };
        if handle == INVALID_SOCKET { return Err(socketerror()); }
        Ok((TcpStream { handle }, SocketAddr::from_sockaddr(addr)))
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        unsafe { closesocket(self.handle); }
    }
}

impl AsSocket for TcpListener {
    fn as_socket(&self) -> Socket {
        self.handle
    }

    fn into_socket(self) -> Socket {
        let me = ManuallyDrop::new(self);
        me.handle
    }

    fn from_socket(socket: Socket) -> TcpListener {
        TcpListener { handle: socket }
    }
}

/// A TCP stream between a local and a remote socket
// TODO: peek, shutdown, connect
pub struct TcpStream {
    handle: Socket,
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe { recv(self.handle, buf.as_mut_ptr(), buf.len() as SendLen, 0) };
        if ret == -1 { return Err(socketerror()); }
        Ok(ret as usize)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { send(self.handle, buf.as_ptr(), buf.len() as SendLen, 0) };
        if ret == -1 { return Err(socketerror()); }
        Ok(ret as usize)
    }
}

impl AsSocket for TcpStream {
    fn as_socket(&self) -> Socket {
        self.handle
    }

    fn into_socket(self) -> Socket {
        let me = ManuallyDrop::new(self);
        me.handle
    }

    fn from_socket(socket: Socket) -> TcpStream {
        TcpStream { handle: socket }
    }
}
