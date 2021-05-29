use lib::fs::ForeachDir;
use lib::libc::ToCString;
use lib::utils::get_args_without_self_path;
use std::fs::{read_dir, DirEntry};
use std::path::Path;

fn main() -> Result<(), ()> {
    let file = "/home/zhc/tmp/a";
    let c_string = file.to_c_string();
    let file_c_str = c_string.as_ptr();
    let i = unsafe {
        libc::chmod(file_c_str, 755) };
    println!("{}, {:?}", i, errno::errno());

    Ok(())
}
