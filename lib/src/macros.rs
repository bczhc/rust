#[macro_export]
macro_rules! rw_read {
    ($e:expr) => {
        $e.read().unwrap()
    };
}

#[macro_export]
macro_rules! rw_write {
    ($e:expr) => {
        $e.write().unwrap()
    };
}

#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}

#[macro_export]
macro_rules! field_size {
    ( $t:ty, $($f:tt),+ $(,)? ) => {
        {
            let mut size = 0_usize;
            $(
                let dummy = ::std::mem::MaybeUninit::uninit();
                let r: &$t = unsafe { dummy.assume_init_ref() };
                size += ::std::mem::size_of_val(&r.$f);
            )*
            size
        }
    }
}
