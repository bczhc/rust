use std::env::args;
use std::ffi::OsString;
use std::path::Path;

pub enum MsgType<'a> {
    Help,
    InvalidArgumentCount(usize),
    UnknownOption(&'a String),
}

pub struct MsgPrinter {
    help_msg: String,
}

impl MsgPrinter {
    pub fn new(help_msg: String) -> MsgPrinter {
        Self { help_msg }
    }

    pub fn show_msg(&self, msg_type: MsgType) -> Result<(), String> {
        match msg_type {
            MsgType::Help => {
                println!("{}", self.help_msg);
                Ok(())
            }
            MsgType::InvalidArgumentCount(count) => {
                Err(format!("Invalid argument count: {}", count))
            }
            MsgType::UnknownOption(option) => Err(format!("Unknown option: {}", option)),
        }
    }
}

pub fn get_file_name() -> String {
    let file_path = args().next().unwrap();
    let file_name = OsString::from(Path::new(&file_path).file_name().unwrap())
        .into_string()
        .unwrap();
    file_name
}

pub fn get_args_without_self_path() -> Vec<String> {
    let args = args();
    let mut c: Vec<String> = args.collect();
    c.remove(0);
    c
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Pair<T1, T2> {
    a: T1,
    b: T2,
}

impl<T1, T2> Pair<T1, T2> {
    #[inline]
    pub fn new(a: T1, b: T2) -> Pair<T1, T2> {
        Self { a, b }
    }
}
