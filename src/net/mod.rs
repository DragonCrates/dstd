//! Networking primitives for TCP/UDP communication

mod sys;

/// Alias for an OS-dependent socket handle
pub use sys::Socket;

#[doc(no_inline)]
pub use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr, AddrParseError};

mod tcp;
pub use tcp::{TcpSocket, TcpListener, TcpStream};

// TODO: gethostname

pub trait AsSocket {
    fn as_socket(&self) -> Socket;
    fn into_socket(self) -> Socket;
    fn from_socket(socket: Socket) -> Self;
}
