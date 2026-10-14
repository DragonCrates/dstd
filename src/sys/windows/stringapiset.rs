use core::ffi::c_int;
use super::minwindef::*;

unsafe extern "system" {
    /// Maps a character string to a UTF-16 (wide character) string.
    pub fn MultiByteToWideChar(
        /* [in] */ CodePage: UINT,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ lpMultiByteStr: LPCCH,
        /* [in] */ cbMultiByte: c_int,
        /* [out, optional */ lpWideCharStr: LPWSTR,
        /* [in] */ cchWideChar: c_int
    ) -> c_int;
}
