fn main() {
    let a: i32 = 2;

    if a == 2 {
        for _i in 0..100 {
            f();
        }
    }
}

fn f() {
    let s: &str = "hello, world";
    println!("{}, {}", s, s);
}