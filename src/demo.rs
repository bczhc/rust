use clap::{App, Arg, ArgGroup};
use std::rc::Rc;

fn main() {
    let x;
    {
        let a = Box::new(2);
        x = a;
    }

    println!("{}", x);
}
