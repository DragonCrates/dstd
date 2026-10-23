use core::ffi::c_int;
use super::minwindef::*;
use super::winnt::*;

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
    /// Maps a UTF-16 (wide character) string to a new character string.
    pub fn WideCharToMultiByte(
        /* [in] */ CodePage: UINT,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ lpWideCharStr: LPCWCH,
        /* [in] */ cchWideChar: c_int,
        /* [out, optional] */ lpMultiByteStr: LPSTR,
        /* [in] */ cbMultiByte: c_int,
        /* [in, optional] */ lpDefaultChar: LPCCH,
        /* [out, optional] */ lpUsedDefaultChar: LPBOOL
    ) -> c_int;
}
