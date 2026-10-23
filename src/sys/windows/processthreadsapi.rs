use super::minwindef::UINT;

unsafe extern "system" {
    /// Ends the calling process and all its threads.
    pub fn ExitProcess(uExitCode: UINT) -> !;
}
