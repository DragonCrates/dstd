use crate::sys::windows::minwindef::UINT;
use crate::sys::windows::processthreadsapi::ExitProcess;

pub fn exit(code: i32) -> ! {
    unsafe { ExitProcess(code as UINT) }
}
