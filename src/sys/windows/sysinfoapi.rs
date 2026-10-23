use super::minwindef::DWORD;

unsafe extern "system" {
    pub fn GetVersion() -> DWORD;
}
