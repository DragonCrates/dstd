//! Networking primitives for TCP/UDP communication

mod sys;

/// Alias for an OS-dependent socket handle
pub use sys::Socket;

#[doc(no_inline)]
pub use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr, AddrParseError};

mod tcp;
pub use tcp::{TcpListener, TcpStream};

// TODO: gethostname
