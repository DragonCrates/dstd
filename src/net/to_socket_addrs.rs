use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr};
use core::option;
use core::slice;
use core::iter;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

use crate::io;

/// A trait for objects which can be converted or resolved to one or more [`SocketAddr`] values
pub trait ToSocketAddrs {
    /// Returned iterator
    type Iter: Iterator<Item = SocketAddr>;

    /// Convert this object to an iterator of resolved [`SocketAddr`]s
    ///
    /// This function may block the current thread
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // Try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<IpAddr>() {
            let addr = SocketAddr::new(addr, port);
            return Ok(vec![addr].into_iter());
        }

        // Otherwise, make the system look it up.
        lookup_host(host, port).map(|addrs| Vec::from_iter(addrs).into_iter())
    }
}

impl ToSocketAddrs for (String, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&*self.0, self.1).to_socket_addrs()
    }
}

// accepts strings like 'localhost:12345'
impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // Try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        // Otherwise, make the system look it up.
        lookup_host_string(self).map(|addrs| Vec::from_iter(addrs).into_iter())
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (**self).to_socket_addrs()
    }
}

fn lookup_host(_addr: &str, _port: u16) -> io::Result<vec::IntoIter<SocketAddr>> {
    todo!()
}

fn lookup_host_string(_addr: &str) -> io::Result<vec::IntoIter<SocketAddr>> {
    /*

    // Split the string by ':' and convert the second part to u16...
    let Some((host, port_str)) = addr.rsplit_once(':') else {
        return Err(io::const_error!(io::ErrorKind::InvalidInput, "invalid socket address"));
    };
    let Ok(port) = port_str.parse::<u16>() else {
        return Err(io::const_error!(io::ErrorKind::InvalidInput, "invalid port value"));
    };

    // ... and make the system look up the host.
    crate::sys::net::lookup_host(host, port)

    */

    todo!()
}
