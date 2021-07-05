use std::sync::{Condvar, Mutex};
use std::thread::spawn;

fn main() {
    let count = num_cpus::get();
    println!("count: {}", count);

    for _i in 0..count {
        spawn(|| loop {});
    }

    let mutex = Mutex::new(0);
    let _guard = Condvar::new().wait(mutex.lock().unwrap()).unwrap();
}
