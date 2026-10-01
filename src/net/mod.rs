//! Networking primitives for TCP/UDP communication

mod sys;

/// Alias for an OS-dependent socket handle
pub use sys::RawSocket;

#[doc(no_inline)]
pub use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr, AddrParseError};

mod to_socket_addrs;
pub use to_socket_addrs::ToSocketAddrs;
mod tcp;
pub use tcp::{TcpSocket, TcpListener, TcpStream};

// TODO: gethostname

pub trait AsRawSocket {
    fn as_raw_socket(&self) -> RawSocket;
    fn into_raw_socket(self) -> RawSocket;
    fn from_raw_socket(socket: RawSocket) -> Self;
}

use crate::io::{self, Error};

fn for_each_addr<T>(addrs: impl ToSocketAddrs, mut f: impl FnMut(SocketAddr) -> io::Result<T>) -> io::Result<T> {
    let mut last_error = Error::new_no_addresses();
    for addr in addrs.to_socket_addrs()? {
        match f(addr) {
            Ok(ret) => return Ok(ret),
            Err(err) => last_error = err,
        }
    }
    Err(last_error)
}
