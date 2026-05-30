//! Networking primitives for TCP/UDP communication

use core::net::SocketAddr;
use core::mem;
use core::ffi::c_int;

use crate::io::{Read, Write, Result, Error};

mod sys;
use sys::*;

/// Alias for an OS-dependent socket handle
pub use sys::Socket;

// TODO:
// - as raw fd
// - set nonblocking
// - getaddrinfo

#[cfg(windows)]
fn init() {
    use crate::sync::Once;
    static WSA_INITIALIZED: Once = Once::new();

    WSA_INITIALIZED.call_once(|| unsafe {
        let mut data: WSADATA = mem::zeroed();
        if WSAStartup(0x0202, &mut data) != 0 {
            panic!("WinSock initialization failed");
        }
    });
}

#[cfg(unix)]
fn init() {}

fn sockerror() -> Error {
    #[cfg(unix)]
    return Error::last_os_error();
    #[cfg(windows)]
    return Error::from_raw_os_error(unsafe { WSAGetLastError() as crate::io::ErrorOs })
}

/// A TCP socket server, listening for connections
pub struct TcpListener {
    handle: Socket,
}

impl TcpListener {
    /// Creates a new TcpListener which will be bound to the specified address
    pub fn bind(addr: SocketAddr) -> Result<TcpListener> {
        init();

        unsafe {
            let handle = socket(addr.address_family() as c_int, SOCK_STREAM, 0);
            if handle == INVALID_SOCKET { return Err(sockerror()); }
            let listener = TcpListener { handle };

            let sockaddr = addr.to_sockaddr();
            let ret = bind(handle, &sockaddr as *const sockaddr, mem::size_of::<sockaddr>() as socklen_t);
            if ret == -1 { return Err(sockerror()); }
            let ret = listen(handle, SOMAXCONN);
            if ret == -1 { return Err(sockerror()); }
            Ok(listener)
        }
    }

    /// Accept a new incoming connection from this listener
    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let mut addr = sockaddr::default();
        let mut addrlen = mem::size_of::<sockaddr>() as socklen_t;
        let handle = unsafe { accept(self.handle, &mut addr, &mut addrlen) };
        if handle == INVALID_SOCKET { return Err(sockerror()); }
        Ok((TcpStream { handle }, SocketAddr::from_sockaddr(addr)))
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        unsafe {
            #[cfg(windows)]
            closesocket(self.handle);
            #[cfg(unix)]
            crate::io::close(self.handle);
        }
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
        if ret == -1 { return Err(sockerror()); }
        Ok(ret as usize)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { send(self.handle, buf.as_ptr(), buf.len() as SendLen, 0) };
        if ret == -1 { return Err(sockerror()); }
        Ok(ret as usize)
    }
}

// TODO: gethostname
