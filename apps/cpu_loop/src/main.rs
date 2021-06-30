use std::thread::{spawn, Thread};
use std::sync::{Mutex, Condvar};

fn main() {
    let count = num_cpus::get();
    println!("count: {}", count);

    for _i in 0..count {
        spawn(|| loop {});
    }

    Condvar::new().wait(Mutex::new(0).lock().unwrap());
}
