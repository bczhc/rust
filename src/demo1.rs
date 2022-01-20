use threadpool::ThreadPool;

fn main() {
    let pool = ThreadPool::new(2);
    for i in 0..100 {
        pool.execute(move || {
            println!("{}", i);
        });
    }
    pool.join();
}