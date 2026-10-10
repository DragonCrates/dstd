use core::ffi::{c_int, c_void};
use core::mem::{self, ManuallyDrop};
use core::net::SocketAddr;
use core::ptr;

use crate::sys::libc::Fd;
use crate::sys::libc::socket::*;
use crate::sys::libc::unistd::close;
use crate::sys::addr::{sockaddr, SocketAddrExt};
use crate::io::{self, Error, Read, Write};
use crate::net::AsRawSocket;
use crate::os_str::OsStr;

pub type RawSocket = Fd;

pub use crate::sys::libc::socket::{SOCK_STREAM, SOCK_DGRAM};

pub struct Socket {
    fd: Fd,
}

impl Socket {
    pub fn new(domain: c_int, socket_type: c_int) -> io::Result<Socket> {
        let fd = unsafe { socket(domain, socket_type | SOCK_CLOEXEC, 0) };
        if fd == -1 { return Err(Error::last_os_error()); }
        let this = Socket { fd };
        this.set_reuseaddr()?;
        Ok(this)
    }

    pub fn new_nonblock(domain: c_int, socket_type: c_int) -> io::Result<Socket> {
        Socket::new(domain, socket_type | SOCK_NONBLOCK)
    }

    /// This allows to quickly reuse released ports without getting EADDRINUSE
    fn set_reuseaddr(&self) -> io::Result<()> {
        let optval: i32 = 1;
        let ret = unsafe { setsockopt(self.fd, SOL_SOCKET, SO_REUSEADDR, &optval as *const _ as *const c_void, 4) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(())
    }

    pub fn bind(&self, addr: SocketAddr) -> io::Result<()> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { bind(self.fd, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(())
    }

    pub fn listen(&self) -> io::Result<()> {
        let ret = unsafe { listen(self.fd, SOMAXCONN) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(())
    }

    pub fn accept(&self, nonblock: bool) -> io::Result<(Socket, SocketAddr)> {
        let nonblock_opt = if nonblock { SOCK_NONBLOCK } else { 0 };

        let mut addr = sockaddr::default();
        let mut addrlen = mem::size_of::<sockaddr>() as socklen_t;

        let fd = unsafe { accept4(self.fd, &mut addr, &mut addrlen, SOCK_CLOEXEC | nonblock_opt) };
        if fd == -1 { return Err(Error::last_os_error()); }

        Ok((Socket { fd }, SocketAddr::from_sockaddr(addr)))
    }

    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { connect(self.fd, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(())
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { send(self.fd, buf.as_ptr(), buf.len(), 0) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { sendto(self.fd, buf.as_ptr(), buf.len(), 0, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        // TODO MSG_NOSIGNAL
        let ret = unsafe { recv(self.fd, buf.as_mut_ptr(), buf.len(), 0) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut sockaddr = sockaddr::default();
        let ret = unsafe { recvfrom(self.fd, buf.as_mut_ptr(), buf.len(), 0, &mut sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok((ret as usize, SocketAddr::from_sockaddr(sockaddr)))
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }
}

impl AsRawSocket for Socket {
    fn as_raw_socket(&self) -> RawSocket {
        self.fd
    }

    fn into_raw_socket(self) -> RawSocket {
        let me = ManuallyDrop::new(self);
        me.fd
    }

    fn from_raw_socket(socket: RawSocket) -> Socket {
        Socket { fd: socket }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe { close(self.fd); }
    }
}

pub struct AddrInfo {
    first: *const addrinfo,
    next: *const addrinfo,
    port: u16,
}

#[allow(clippy::field_reassign_with_default)]
pub fn lookup_host(addr: &str, port: u16) -> io::Result<AddrInfo> {
    let mut addrbuf = [0; 256];
    let c_addr = OsStr::from_str_with(addr, &mut addrbuf).unwrap();

    let mut hints = addrinfo::default();
    hints.ai_socktype = SOCK_STREAM;
    let mut result = ptr::null_mut();

    let ret = unsafe { getaddrinfo(c_addr.as_ptr(), ptr::null(), &hints, &mut result) };
    if ret != 0 { return Err(Error::new_addrinfo(ret)); }

    Ok(AddrInfo {
        first: result,
        next: result,
        port
    })
}

impl Iterator for AddrInfo {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        if self.next.is_null() {
            None
        } else {
            let next = unsafe { &*self.next };
            let sockaddr = unsafe { *next.ai_addr };
            let mut addr = SocketAddr::from_sockaddr(sockaddr);
            addr.set_port(self.port);
            self.next = next.ai_next;
            Some(addr)
        }
    }
}

impl Drop for AddrInfo {
    fn drop(&mut self) {
        unsafe { freeaddrinfo(self.first as *mut _); }
    }
}
