use lib::libc::ToCString;

fn main() -> Result<(), ()> {
    let file = "/home/zhc/tmp/a";
    let c_string = file.to_c_string();
    let file_c_str = c_string.as_ptr();
    let i = unsafe { libc::chmod(file_c_str, 755) };
    println!("{}, {:?}", i, errno::errno());

    Ok(())
}
