use super::minwindef::*;

pub type PFLS_CALLBACK_FUNCTION = unsafe extern "system" fn(
    /* [in] */ lpFlsData: PVOID,
);
unsafe extern "system" {
    /// Allocates a fiber local storage (FLS) index
    pub fn FlsAlloc(
        /* [in] */ lpCallback: PFLS_CALLBACK_FUNCTION,
    ) -> DWORD;
    /// Retrieves the value in the calling fiber's FLS slot for the specified FLS index
    pub fn FlsGetValue(
        /* [in] */ dwFlsIndex: DWORD
    ) -> PVOID;
    /// Stores a value in the calling fiber's FLS slot for the specified FLS index
    pub fn FlsSetValue(
        /* [in] */ dwFlsIndex: DWORD,
        /* [in, optional] */ lpFlsData: PVOID,
    ) -> BOOL;
}

pub const FLS_OUT_OF_INDEXES: DWORD = 0xffffffff;
