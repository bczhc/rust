use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn get_current_time_millis() -> u64 {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    t.as_millis() as u64
}

pub fn current_time_millis() -> u64 {
    get_current_time_millis()
}
