use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;

/// Variable lazy delegate
/// # Examples
/// ```
/// use bczhc_lib::lazy::Lazy;
///
/// let s = Lazy::new(|| String::from("lazy string"));
/// // initializing when de-referenced
/// assert_eq!(&*s, "lazy string")
/// ```
pub struct Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    ptr: *mut *mut T,
    initializer: F,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    pub fn new(initializer: F) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(null_mut())),
            initializer,
        }
    }
}

impl<T, F> Deref for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if (*self.ptr).is_null() {
                *self.ptr = Box::into_raw(Box::new((self.initializer)()));
            }
            &**self.ptr
        }
    }
}

impl<T, F> DerefMut for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            if (*self.ptr).is_null() {
                *self.ptr = Box::into_raw(Box::new((self.initializer)()));
            }
            &mut **self.ptr
        }
    }
}

impl<T, F> Drop for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    fn drop(&mut self) {
        unsafe {
            let obj_ptr = *self.ptr;
            drop(Box::from_raw(obj_ptr));
            drop(Box::from_raw(self.ptr as *mut *mut T));
        }
    }
}
