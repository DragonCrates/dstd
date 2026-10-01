use super::minwindef::*;

unsafe extern "C" {
    /// Creates or opens a file or I/O device.
    pub fn CreateFileW(
        /* [in] */ lpFileName: LPCWSTR,
        /* [in] */ dwDesiredAccess: DWORD,
        /* [in] */ dwShareMode: DWORD,
        /* [in, optional] */ lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        /* [in] */ dwCreationDisposition: DWORD,
        /* [in] */ dwFlagsAndAttributes: DWORD,
        /* [in, optional] */ hTemplateFile: HANDLE,
    ) -> HANDLE;

    /// Reads data from the specified file or input/output (I/O) device.
    pub fn ReadFile(
        /* [in] */ hFile: HANDLE,
        /* [out] */ lpBuffer: LPVOID,
        /* [in] */ nNumberOfBytesToRead: DWORD,
        /* [out, optional] */ lpNumberOfBytesRead: LPDWORD,
        /* [in, out, optional] */ lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    /// Writes data to the specified file or input/output (I/O) device.
    pub fn WriteFile(
        /* [in] */ hFile: HANDLE,
        /* [in] */ lpBuffer: LPCVOID,
        /* [in] */ nNumberOfBytesToWrite: DWORD,
        /* [out, optional] */ lpNumberOfBytesWritten: LPDWORD,
        /* [in, out, optional] */ lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    /// Moves the file pointer of the specified file.
    pub fn SetFilePointerEx(
        /* [in] */ hFile: HANDLE,
        /* [in] */ liDistanceToMove: LARGE_INTEGER,
        /* [out, optional] */ lpNewFilePointer: PLARGE_INTEGER,
        /* [in] */ dwMoveMethod: DWORD,
    ) -> BOOL;
}
