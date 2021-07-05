use std::borrow::BorrowMut;
use std::cell::RefCell;

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

fn main() {
    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let count = 10;

    let handlers = Arc::new(RefCell::new(Vec::new()));

    for i in 0..count {
        let moved_pair = pair.clone();
        let moved_handlers = handlers.clone();

        let handle = spawn(move || {
            let (mutex, cond) = &*moved_pair;
            cond.wait(mutex.lock().unwrap());
            println!("{}", i);
        });

        let ref_cell = &*moved_handlers;
        let handlers = &mut *ref_cell.borrow_mut();
        handlers.push(handle);
    }

    println!("start");

    sleep(Duration::from_secs(2));
    println!("notify all");
    (*pair).1.notify_all();

    let handlers = (*handlers).take();
    for handler in handlers {
        handler.join().unwrap();
    }
}
