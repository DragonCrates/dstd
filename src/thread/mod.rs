use core::cell::UnsafeCell;
use core::time::Duration;

use crate::sync::Arc;
use crate::prelude::Box;

#[cfg(unix)]
crate::block! {
    mod pthread;
    use pthread as sys;
}

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

/// Creates a new thread
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let ret = Arc::new(UnsafeCell::new(None));
    let ret2 = Arc::clone(&ret);

    let args = ThreadInit { thread_main: f, ret: ret2 };
    let boxed_args = Box::new(args);
    let args_ptr = Box::into_raw(boxed_args);

    let handle = sys::spawn(args_ptr);
    JoinHandle { handle, ret }
}

pub(crate) struct ThreadInit<F, T> {
    thread_main: F,
    ret: Arc<UnsafeCell<Option<T>>>,
}

pub struct JoinHandle<T> {
    handle: sys::JoinHandle,
    ret: Arc<UnsafeCell<Option<T>>>,
}

impl<T> JoinHandle<T> {
    /// Wait for the associated thread to finish
    /// # Panics
    /// This function panics if it can't retrieve thread's return value (meaning it did not finish), for example when it was cancelled by native code
    pub fn join(self) -> T {
        self.handle.join();
        let ret = unsafe { &mut *self.ret.get() };
        ret.take().expect("thread was cancelled")
    }

    // TODO: as handle, as pthread_t
}

/// Returns the amount of available processor cores
///
/// Corresponds to `sysconf(_SC_NPROCESSORS_ONLN)` on Unix
pub fn available_parallelism() -> usize {
    sys::ncpu()
}

pub fn sleep(dur: Duration) {
    crate::time::sleep(dur);
}

pub fn usleep(ms: u64) {
    sleep(Duration::from_millis(ms));
}

// TODO: thread names
