mod sys {
    unsafe extern "C" {
        pub fn exit(code: i32) -> !;
    }
}

pub fn exit(code: i32) -> ! {
    unsafe { sys::exit(code) }
}
