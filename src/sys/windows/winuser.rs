use core::ffi::c_int;
use super::minwindef::*;

unsafe extern "C" {
    /// Displays a modal dialog box that contains a system icon, a set of buttons, and a brief application-specific message, such as status or error information.
    pub fn MessageBoxW(
        /* [in, optional] */ hWnd: HWND,
        /* [in, optional] */ lpText: LPCWSTR,
        /* [in, optional] */ lpCaption: LPCWSTR,
        /* [in] */ uType: UINT,
    ) -> c_int;
}

pub const MB_ICONERROR: UINT = 0x00000010;
