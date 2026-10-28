use core::fmt;

macro_rules! impl_from_primitive {
    (
        $(#[$attr:meta])*
        $v:vis enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$attr])*
        $v enum $name {
            $($variant),*
        }

        impl $name {
            pub(crate) const fn from_primitive(n: usize) -> Option<$name> {
                match n {
                    $(v if v == $name::$variant as usize => Some($name::$variant),)*
                    _ => None,
                }
            }
        }
    }
}

impl_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ErrorKind {
        // dstd errors
        UnexpectedEof,
        WriteZero,
        InvalidInput,
        InvalidData,

        NotFound,
        PermissionDenied,
        // network errors
        ConnectionRefused,
        ConnectionReset,
        HostUnreachable,
        NetworkUnreachable,
        ConnectionAborted,
        NotConnected,
        AddrInUse,
        AddrNotAvailable,
        NetworkDown,

        BrokenPipe,
        AlreadyExists,
        Interrupted,
        WouldBlock,
        NotADirectory,
        TimedOut,

        AddrInfo,
        Other,
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // dstd errors
            ErrorKind::UnexpectedEof => f.write_str("unexpected eof"),
            ErrorKind::WriteZero => f.write_str("write zero"),
            ErrorKind::InvalidInput => f.write_str("invalid input"),
            ErrorKind::InvalidData => f.write_str("invalid data"),

            ErrorKind::NotFound => f.write_str("not found"),
            ErrorKind::PermissionDenied => f.write_str("permission denied"),
            // network errors
            ErrorKind::ConnectionRefused => f.write_str("connection refused"),
            ErrorKind::ConnectionReset => f.write_str("connection reset"),
            ErrorKind::HostUnreachable => f.write_str("host unreachable"),
            ErrorKind::NetworkUnreachable => f.write_str("network unreachable"),
            ErrorKind::ConnectionAborted => f.write_str("connection aborted"),
            ErrorKind::NotConnected => f.write_str("not connected"),
            ErrorKind::AddrInUse => f.write_str("addr in use"),
            ErrorKind::AddrNotAvailable => f.write_str("addr not available"),
            ErrorKind::NetworkDown => f.write_str("network down"),

            ErrorKind::BrokenPipe => f.write_str("broken pipe"),
            ErrorKind::AlreadyExists => f.write_str("already exists"),
            ErrorKind::Interrupted => f.write_str("interrupted"),
            ErrorKind::WouldBlock => f.write_str("would block"),
            ErrorKind::NotADirectory => f.write_str("not a directory"),
            ErrorKind::TimedOut => f.write_str("timed out"),

            ErrorKind::AddrInfo => f.write_str("getaddrinfo failed"),
            ErrorKind::Other => f.write_str("other"),
        }
    }
}

impl core::error::Error for ErrorKind {}
