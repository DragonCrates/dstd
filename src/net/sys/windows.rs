use core::mem::{self, ManuallyDrop};
use core::ptr;
use core::ffi::{c_int, c_ulong};
use core::net::SocketAddr;

use crate::sys::windows::minwindef::*;
use crate::sys::windows::winsock2::*;
use crate::sys::addr::{sockaddr, SocketAddrExt};
use crate::io::{self, Error, Read, Write};
use crate::net::AsRawSocket;
use crate::os_str::OsStr;

pub type RawSocket = SOCKET;

pub use crate::sys::windows::winsock2::{SOCK_STREAM, SOCK_DGRAM};

fn wsa_startup() {
    use crate::sync::Once;
    static WSA_INITIALIZED: Once = Once::new();

    WSA_INITIALIZED.call_once(|| unsafe {
        let mut data: WSADATA = mem::zeroed();
        if WSAStartup(0x0202, &mut data) != 0 {
            panic!("WinSock initialization failed");
        }
    });
}

pub fn sockerror() -> Error {
    Error::from_raw_os_error(unsafe { WSAGetLastError() as DWORD })
}

pub struct Socket {
    handle: SOCKET,
}

impl Socket {
    pub fn new(domain: c_int, socket_type: c_int) -> io::Result<Socket> {
        wsa_startup();
        let handle = unsafe { WSASocketW(domain, socket_type, 0, ptr::null_mut(), 0, WSA_FLAG_NO_HANDLE_INHERIT) };
        if handle == INVALID_SOCKET { return Err(sockerror()); }
        Ok(Socket { handle })
    }

    pub fn new_nonblock(domain: c_int, socket_type: c_int) -> io::Result<Socket> {
        let sock = Socket::new(domain, socket_type)?;
        sock.set_nonblock(true)?;
        Ok(sock)
    }

    fn set_nonblock(&self, nonblock: bool) -> io::Result<()> {
        let mut mode = nonblock as c_ulong;
        let ret = unsafe { ioctlsocket(self.handle, FIONBIO, &mut mode) };
        if ret == SOCKET_ERROR { return Err(sockerror()); }
        Ok(())
    }

    // TODO: everything below may be merged with unix part
    pub fn bind(&self, addr: SocketAddr) -> io::Result<()> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { bind(self.handle, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(sockerror()); }
        Ok(())
    }

    pub fn listen(&self) -> io::Result<()> {
        let ret = unsafe { listen(self.handle, SOMAXCONN) };
        if ret == -1 { return Err(sockerror()); }
        Ok(())
    }

    pub fn accept(&self, nonblock: bool) -> io::Result<(Socket, SocketAddr)> {
        let mut addr = sockaddr::default();
        let mut addrlen = mem::size_of::<sockaddr>() as socklen_t;

        let handle = unsafe { accept(self.handle, &mut addr, &mut addrlen) };
        if handle == INVALID_SOCKET { return Err(sockerror()); }
        let sock = Socket { handle };
        if nonblock {
            sock.set_nonblock(true)?;
        }

        Ok((sock, SocketAddr::from_sockaddr(addr)))
    }

    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { connect(self.handle, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(())
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { send(self.handle, buf.as_ptr(), buf.len() as c_int, 0) };
        if ret == -1 { return Err(sockerror()); }
        Ok(ret as usize)
    }

    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let sockaddr = addr.to_sockaddr();
        let ret = unsafe { sendto(self.handle, buf.as_ptr(), buf.len() as c_int, 0, &sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { recv(self.handle, buf.as_mut_ptr(), buf.len() as c_int, 0) };
        if ret == -1 { return Err(sockerror()); }
        Ok(ret as usize)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut sockaddr = sockaddr::default();
        let ret = unsafe { recvfrom(self.handle, buf.as_mut_ptr(), buf.len() as c_int, 0, &mut sockaddr, mem::size_of::<sockaddr>() as socklen_t) };
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
        self.handle
    }

    fn into_raw_socket(self) -> RawSocket {
        let me = ManuallyDrop::new(self);
        me.handle
    }

    fn from_raw_socket(socket: RawSocket) -> Socket {
        Socket { handle: socket }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe { closesocket(self.handle); }
    }
}

pub struct AddrInfo {
    first: *const ADDRINFOW,
    next: *const ADDRINFOW,
    port: u16,
}

pub fn lookup_host(addr: &str, port: u16) -> io::Result<AddrInfo> {
    let mut addrbuf = [0; 256];
    let c_addr = OsStr::from_str_with(addr, &mut addrbuf).unwrap();
    let hints = ADDRINFOW::default();
    let mut result = ptr::null_mut();
    let ret = unsafe { GetAddrInfoW(c_addr.as_ptr(), ptr::null(), &hints, &mut result) };
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
        unsafe { FreeAddrInfoW(self.first as *mut _); }
    }
}
