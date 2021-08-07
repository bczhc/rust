use clap::{App, Arg, ArgGroup};
use std::rc::Rc;
use std::io::stdin;
use lib::io::ReadLine;

fn main() {
    let s = stdin().read_line_without_line_terminator().unwrap();

    println!("{}", s.len());
}
