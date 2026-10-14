use super::minwindef::*;

unsafe extern "system" {
    /// Retrieves a handle to the specified standard device (standard input, standard output, or standard error).
    pub fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
}
