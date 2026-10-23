use super::minwindef::*;

unsafe extern "system" {
    /// Retrieves the current input mode of a console's input buffer or the current output mode of a console screen buffer.
    pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
    /// Sets the input code page used by the console associated with the calling process.
    pub fn SetConsoleCP(/*_In_*/ wCodePageID: UINT) -> BOOL;
    /// Retrieves the input code page used by the console associated with the calling process.
    pub fn GetConsoleCP() -> UINT;
    /// Sets the output code page used by the console associated with the calling process.
    pub fn SetConsoleOutputCP(/*_In_*/ wCodePageID: UINT) -> BOOL;
    /// Retrieves the output code page used by the console associated with the calling process.
    pub fn GetConsoleOutputCP() -> UINT;
    /// Reads character input from the console input buffer and removes it from the buffer.
    pub fn ReadConsoleW(
        /* _In_     */ hConsoleInput: HANDLE,
        /* _Out_    */ lpBuffer: LPVOID,
        /* _In_     */ nNumberOfCharsToRead: DWORD,
        /* _Out_    */ lpNumberOfCharsRead: LPDWORD,
        /* _In_opt_ */ pInputControl: LPVOID
    ) -> BOOL;
    /// Writes a character string to a console screen buffer beginning at the current cursor location.
    pub fn WriteConsoleW(
        /* _In_ */ hConsoleOutput: HANDLE,
        /* _In_ */ lpBuffer: LPCVOID,
        /* _In_ */ nNumberOfCharsToWrite: DWORD,
        /* _Out_opt_  */ lpNumberOfCharsWritten: LPDWORD,
        /* _Reserved_ */ lpReserved: LPVOID,
    ) -> BOOL;
}
