fn main() {
    struct A {}
    struct B {}
    trait I {
        fn f(&self);
    }

    impl I for A {
        fn f(&self) {
            println!("A");
        }
    }
    impl I for B {
        fn f(&self) {
            println!("B");
        }
    }

    let o: &I;
    let a = A {};
    let b = B {};

    if 1 == 1 {
        o = &a;
    } else {
        o = &b;
    }

    o.f();
}