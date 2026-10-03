use std::ffi::{c_char, CStr, CString};

#[no_mangle]
pub extern "C" fn astro_abi_version() -> u32 {
    1
}

/// # Safety
/// `input` must point to a readable NUL-terminated string, or be null.
/// The returned pointer is owned by Rust and must be freed with astro_free_string once.
#[no_mangle]
pub unsafe extern "C" fn astro_calculate_json(input: *const c_char) -> *mut c_char {
    invoke_json(input, astro_core::calculate_json)
}

/// # Safety
/// `input` must point to a readable NUL-terminated JSON wrapper with payload/options, or be null.
/// The returned pointer is owned by Rust and must be freed with astro_free_string once.
#[no_mangle]
pub unsafe extern "C" fn astro_compress_json(input: *const c_char) -> *mut c_char {
    invoke_json(input, astro_core::compress_json)
}

/// # Safety
/// `input` must point to a readable NUL-terminated context envelope, or be null.
/// The returned pointer is owned by Rust and must be freed with astro_free_string once.
#[no_mangle]
pub unsafe extern "C" fn astro_expand_context_json(input: *const c_char) -> *mut c_char {
    invoke_json(input, astro_core::expand_context_json)
}

unsafe fn invoke_json(input: *const c_char, operation: fn(&str) -> String) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if input.is_null() {
            return astro_core::error_json("INVALID_INPUT", "Null request pointer");
        }
        match CStr::from_ptr(input).to_str() {
            Ok(text) => operation(text),
            Err(_) => astro_core::error_json("INVALID_INPUT", "Input must be UTF-8"),
        }
    })
    .unwrap_or_else(|_| {
        astro_core::error_json("INTERNAL_ERROR", "Engine panic contained at FFI boundary")
    });
    CString::new(result)
        .expect("Serialized JSON contains no literal NUL")
        .into_raw()
}

/// # Safety
/// Pointer must be null or an unfreed value returned by an astro_*_json function.
#[no_mangle]
pub unsafe extern "C" fn astro_free_string(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(CString::from_raw(pointer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_request_returns_owned_error() {
        unsafe {
            for operation in [
                astro_calculate_json,
                astro_compress_json,
                astro_expand_context_json,
            ] {
                let result = operation(std::ptr::null());
                assert!(CStr::from_ptr(result)
                    .to_str()
                    .unwrap()
                    .contains("INVALID_INPUT"));
                astro_free_string(result);
            }
            astro_free_string(std::ptr::null_mut());
        }
    }
}
