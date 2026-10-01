use super::minwindef::*;

unsafe extern "C" {
    /// Retrieves the current input mode of a console's input buffer or the current output mode of a console screen buffer.
    pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
    /// Writes a character string to a console screen buffer beginning at the current cursor location.
    pub fn WriteConsoleW(
        /* _In_ */ hConsoleOutput: HANDLE,
        /* _In_ */ lpBuffer: LPCVOID,
        /* _In_ */ nNumberOfCharsToWrite: DWORD,
        /* _Out_opt_ */ lpNumberOfCharsWritten: LPDWORD,
        /* _Reserved_ */ lpReserved: LPVOID,
    ) -> BOOL;
}
