use super::minwindef::DWORD;

pub const WSABASEERR: DWORD = 10000;
pub const WSAEINTR: DWORD = WSABASEERR + 4;
pub const WSAEWOULDBLOCK: DWORD = WSABASEERR + 35;
