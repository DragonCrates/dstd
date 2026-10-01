use super::minwindef::*;
use super::wincontypes::*;

#[repr(C)]
#[derive(Default)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub dwSize: COORD,
    pub dwCursorPosition: COORD,
    pub wAttributes: WORD,
    pub srWindow: SMALL_RECT,
    pub dwMaximumWindowSize: COORD,
}
pub type PCONSOLE_SCREEN_BUFFER_INFO = *mut CONSOLE_SCREEN_BUFFER_INFO;

unsafe extern "C" {
    /// Sets the attributes of characters written to the console screen buffer by the WriteFile or WriteConsole function, or echoed by the ReadFile or ReadConsole function. This function affects text written after the function call.
    pub fn SetConsoleTextAttribute(
        /* _In_ */ hConsoleOutput: HANDLE,
        /* _In_ */ wAttributes: WORD,
    ) -> BOOL;
    /// Retrieves information about the specified console screen buffer.
    pub fn GetConsoleScreenBufferInfo(
        /* _In_ */ hConsoleOutput: HANDLE,
        /* _Out_ */ lpConsoleScreenBufferInfo: PCONSOLE_SCREEN_BUFFER_INFO,
    ) -> BOOL;
}
