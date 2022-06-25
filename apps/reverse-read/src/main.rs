use clap::{Arg, Command};
use std::fs::File;
use std::io::{stdout, Read, Seek, SeekFrom, Write};

fn main() {
    let matches = Command::new("rev-read")
        .about("Read file backwards byte-by-byte")
        .arg(Arg::new("file").required(true))
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let mut file = File::open(file).unwrap();
    let mut stdout = stdout().lock();

    let file_size = file.seek(SeekFrom::End(0)).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    if file_size == 0 {
        panic!("File size is zero");
    }

    macro_rules! read_and_output {
        ($file:expr, $stdout:expr) => {
            let mut buf = [0_u8; 1];
            file.read_exact(&mut buf).unwrap();
            stdout.write_all(&buf).unwrap();
            stdout.flush().unwrap();
        };
    }

    file.seek(SeekFrom::End(-1)).unwrap();
    read_and_output!(file, stdout);
    if file_size == 1 {
        return;
    }
    loop {
        file.seek(SeekFrom::Current(-2)).unwrap();
        read_and_output!(file, stdout);
        if file.stream_position().unwrap() == 1 {
            return;
        }
    }
}
