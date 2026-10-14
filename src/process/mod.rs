#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

pub fn exit(code: i32) -> ! {
    unsafe { sys::exit(code) }
}
