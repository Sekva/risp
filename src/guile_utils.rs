#[macro_export]
macro_rules! register_args_function {
    ($f_name:expr, $obg:expr, $f:expr) => {
        let ___fn_name: *const std::os::raw::c_char = std::ffi::CStr::from_bytes_with_nul($f_name)
            .unwrap()
            .as_ptr();
        let ___function: *mut std::os::raw::c_void = $f as *mut std::os::raw::c_void;
        unsafe {
            scm_c_define_gsubr(___fn_name, $obg, 0, 0, ___function);
        }
    };
}
