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
