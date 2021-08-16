use std::ffi::CString;

pub trait ToCString {
    /// As a string pointer in C, for libc FFI.
    ///
    /// # Examples
    /// ```
    /// use bczhc_lib::libc::ToCString;
    /// let heap_str = String::from("abc");
    /// let stack_str = "cba";
    ///
    /// heap_str.to_c_string().as_ptr();
    /// stack_str.to_c_string().as_ptr();
    /// ```
    fn to_c_string(&self) -> CString;
}

impl ToCString for String {
    fn to_c_string(&self) -> CString {
        CString::new(self.as_str()).unwrap()
    }
}

impl ToCString for str {
    fn to_c_string(&self) -> CString {
        CString::new(self).unwrap()
    }
}
