use std::env::args;
use std::sync::{Condvar, Mutex};
use std::thread::spawn;

fn main() {
    let args = args().skip(1);
    let args: Vec<String> = args.collect();

    let mut count = num_cpus::get();

    if args.len() == 1 {
        count = args[0].parse().unwrap();
    }

    println!("count: {}", count);

    for _i in 0..count {
        #[allow(clippy::empty_loop)]
        spawn(|| loop {});
    }

    let mutex = Mutex::new(0);
    let _guard = Condvar::new().wait(mutex.lock().unwrap()).unwrap();
}
