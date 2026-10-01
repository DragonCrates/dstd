use super::minwindef::*;

pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE; // -1

unsafe extern "C" {
    /// Closes an open object handle.
    pub fn CloseHandle(
        /* [in] */ hObject: HANDLE
    ) -> BOOL;
}
