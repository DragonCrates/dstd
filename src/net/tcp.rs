use core::mem;
use core::net::SocketAddr;
use core::ffi::c_int;

use crate::io::{Read, Write, Result};

use super::sys::*;

// TODO:
// - as raw fd
// - set nonblocking
// - getaddrinfo

/// A TCP socket server, listening for connections
pub struct TcpListener {
    handle: Socket,
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified address
    pub fn bind(addr: SocketAddr) -> Result<TcpListener> {
        init();

        unsafe {
            let handle = socket(addr.address_family() as c_int, SOCK_STREAM, 0);
            if handle == INVALID_SOCKET { return Err(socketerror()); }
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

/// A TCP stream between a local and a remote socket
// TODO: peek, shutdown, connect, AsHandle, FromHandle
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
