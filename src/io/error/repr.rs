use core::mem;
use core::ptr::NonNull;

use super::{RawError, ErrorData, SimpleMessage, Custom, CustomData};

pub struct Repr(NonNull<ErrorData>);

const TAG_MASK: usize = 0b11;
const TAG_OS: usize = 0b01;
const TAG_SIMPLE: usize = 0b10;
const TAG_CUSTOM: usize = 0b11;

impl Repr {
    pub fn new_os(os: RawError) -> Repr {
        // This comes from the fact that all Windows and Linux errors are positive
        // On 32-bit architectures, this will eat 2 bits from the end, which is fine as long as errno >= 0
        let tagged = ((os as usize) << 2) | TAG_OS;
        let repr = unsafe { NonNull::new_unchecked(tagged as *mut ErrorData) };
        Repr(repr)
    }

    pub const fn new_simple(msg: &'static SimpleMessage) -> Repr {
        let ptr = msg as *const _ as *mut u8;
        let tagged = ptr.wrapping_add(TAG_SIMPLE);
        unsafe { Repr(NonNull::new_unchecked(tagged as *mut ErrorData)) }
    }

    pub fn new_custom(custom: Custom) -> Repr {
        let ptr = custom.0.as_ptr() as *mut u8;
        let tagged = ptr.wrapping_add(TAG_CUSTOM);
        unsafe { Repr(NonNull::new_unchecked(tagged as *mut ErrorData)) }
    }

    pub fn decode(&self) -> ErrorData {
        let bits = self.0.as_ptr().addr();
        match bits & TAG_MASK {
            TAG_OS => {
                let os = bits >> 2;
                ErrorData::Os(os as RawError)
            }

            TAG_SIMPLE => {
                let tagged = self.0.as_ptr() as *mut u8;
                let ptr = tagged.wrapping_sub(TAG_SIMPLE) as *const SimpleMessage;
                let msg = unsafe { &*ptr };
                ErrorData::Message(msg)
            }

            TAG_CUSTOM => {
                let tagged = self.0.as_ptr() as *mut u8;
                let ptr = tagged.wrapping_sub(TAG_CUSTOM);
                let nonnull = unsafe { NonNull::new_unchecked(ptr as *mut CustomData) };
                ErrorData::Custom(Custom(nonnull))
            }

            _ => unreachable!()
        }
    }
}

macro_rules! static_assert {
    ($cond:expr) => {
        const _: () = assert!($cond);
    };

    (@ $code:expr) => {
        const _: () = $code;
    }
}

static_assert!(mem::align_of::<SimpleMessage>() >= 4);

const fn assert_send<T: Send>() {}
const fn assert_sync<T: Sync>() {}
static_assert!(@ assert_send::<ErrorData>());
static_assert!(@ assert_sync::<ErrorData>());
// SAFETY: ErrorData is Send + Sync
unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}
