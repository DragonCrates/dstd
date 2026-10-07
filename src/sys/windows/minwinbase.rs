use super::minwindef::DWORD;

#[repr(C)]
#[derive(Default)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

pub type LPFILETIME = *mut FILETIME;
