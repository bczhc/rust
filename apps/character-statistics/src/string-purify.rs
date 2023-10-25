use std::io;
use std::io::{stdin, stdout, BufReader, BufWriter};
use utf8_chars::BufReadCharsExt;

fn main() -> io::Result<()> {
    let mut reader = BufReader::new(stdin().lock());
    let mut writer = BufWriter::new(stdout().lock());
    for c in reader.chars().flatten() {
        use io::Write;
        write!(&mut writer, "{}", c)?;
    }
    Ok(())
}
