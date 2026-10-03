//! Minimal Node-API v1 adapter. No Node headers or JS framework dependency.
use std::ffi::{c_char, c_void};
type Handle = *mut c_void;
type Callback = Option<unsafe extern "C" fn(Handle, Handle) -> Handle>;
extern "C" {
    fn napi_get_cb_info(
        env: Handle,
        info: Handle,
        argc: *mut usize,
        argv: *mut Handle,
        this_arg: *mut Handle,
        data: *mut *mut c_void,
    ) -> i32;
    fn napi_get_value_string_utf8(
        env: Handle,
        value: Handle,
        buffer: *mut c_char,
        size: usize,
        result: *mut usize,
    ) -> i32;
    fn napi_create_string_utf8(
        env: Handle,
        text: *const c_char,
        length: usize,
        result: *mut Handle,
    ) -> i32;
    fn napi_create_function(
        env: Handle,
        name: *const c_char,
        length: usize,
        callback: Callback,
        data: *mut c_void,
        result: *mut Handle,
    ) -> i32;
    fn napi_set_named_property(
        env: Handle,
        object: Handle,
        name: *const c_char,
        value: Handle,
    ) -> i32;
    fn napi_throw_type_error(env: Handle, code: *const c_char, message: *const c_char) -> i32;
    fn napi_throw_error(env: Handle, code: *const c_char, message: *const c_char) -> i32;
}

unsafe fn fail(env: Handle, message: &'static [u8]) -> Handle {
    napi_throw_type_error(env, std::ptr::null(), message.as_ptr().cast());
    std::ptr::null_mut()
}

unsafe extern "C" fn calculate(env: Handle, info: Handle) -> Handle {
    invoke_json(
        env,
        info,
        astro_core::calculate_json,
        astro_core::MAX_REQUEST_BYTES,
    )
}

unsafe extern "C" fn compress(env: Handle, info: Handle) -> Handle {
    invoke_json(
        env,
        info,
        astro_core::compress_json,
        astro_core::MAX_CONTEXT_BYTES,
    )
}

unsafe extern "C" fn expand_context(env: Handle, info: Handle) -> Handle {
    invoke_json(
        env,
        info,
        astro_core::expand_context_json,
        astro_core::MAX_CONTEXT_BYTES,
    )
}

unsafe fn invoke_json(
    env: Handle,
    info: Handle,
    operation: fn(&str) -> String,
    max_bytes: usize,
) -> Handle {
    let mut argc = 1;
    let mut argument = std::ptr::null_mut();
    if napi_get_cb_info(
        env,
        info,
        &mut argc,
        &mut argument,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ) != 0
        || argc != 1
    {
        return fail(env, b"Native JSON function requires one JSON string\0");
    }
    let mut size = 0;
    if napi_get_value_string_utf8(env, argument, std::ptr::null_mut(), 0, &mut size) != 0 {
        return fail(env, b"Native JSON function requires a string\0");
    }
    if size > max_bytes {
        return fail(env, b"JSON input exceeds this function's byte limit\0");
    }
    let mut bytes = vec![0u8; size + 1];
    let mut copied = 0;
    if napi_get_value_string_utf8(
        env,
        argument,
        bytes.as_mut_ptr().cast(),
        bytes.len(),
        &mut copied,
    ) != 0
    {
        return fail(env, b"Could not read JSON input\0");
    }
    let input = match std::str::from_utf8(&bytes[..copied]) {
        Ok(input) => input,
        Err(_) => return fail(env, b"Input must be UTF-8\0"),
    };
    if input.contains('\0') {
        return fail(env, b"Input JSON cannot contain a literal NUL\0");
    }
    let output = std::panic::catch_unwind(|| operation(input))
        .unwrap_or_else(|_| astro_core::error_json("INTERNAL_ERROR", "Engine panic contained"));
    let mut result = std::ptr::null_mut();
    if napi_create_string_utf8(env, output.as_ptr().cast(), output.len(), &mut result) != 0 {
        napi_throw_error(
            env,
            std::ptr::null(),
            c"Could not create JSON result".as_ptr(),
        );
        return std::ptr::null_mut();
    }
    result
}

/// # Safety
/// Called by Node with valid environment and exports handles.
#[no_mangle]
pub unsafe extern "C" fn napi_register_module_v1(env: Handle, exports: Handle) -> Handle {
    for (name, callback) in [
        (
            b"calculateJson\0".as_slice(),
            calculate as unsafe extern "C" fn(Handle, Handle) -> Handle,
        ),
        (
            b"compressJson\0".as_slice(),
            compress as unsafe extern "C" fn(Handle, Handle) -> Handle,
        ),
        (
            b"expandContextJson\0".as_slice(),
            expand_context as unsafe extern "C" fn(Handle, Handle) -> Handle,
        ),
    ] {
        let mut function = std::ptr::null_mut();
        if napi_create_function(
            env,
            name.as_ptr().cast(),
            name.len() - 1,
            Some(callback),
            std::ptr::null_mut(),
            &mut function,
        ) != 0
            || napi_set_named_property(env, exports, name.as_ptr().cast(), function) != 0
        {
            napi_throw_error(
                env,
                std::ptr::null(),
                c"Could not initialize astrology addon".as_ptr(),
            );
            return std::ptr::null_mut();
        }
    }
    exports
}
