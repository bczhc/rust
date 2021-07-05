use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read};
use std::path::Path;

pub trait ReadLine {
    fn read_line_without_line_terminator(&mut self) -> Option<String>;
}

impl<T> ReadLine for T
where
    T: Read,
{
    fn read_line_without_line_terminator(&mut self) -> Option<String> {
        let mut read: Vec<u8> = Vec::new();
        let mut buf = [0_u8];
        loop {
            let result = self.read_exact(&mut buf);
            if let Err(e) = result {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    return None;
                } else {
                    panic!("{}", e.to_string());
                }
            }
            if buf[0] == b'\n' {
                break;
            }
            read.push(buf[0]);
        }
        return Some(String::from_utf8(read).unwrap());
    }
}

pub trait OpenOrCreate {
    fn open_or_create<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .read(true)
            .open(path.as_ref())
    }
}

impl OpenOrCreate for File {}

pub trait Skip {
    fn skip(&mut self, size: usize) -> std::io::Result<()>;
}

impl<T> Skip for T
where
    T: Read,
{
    fn skip(&mut self, size: usize) -> std::io::Result<()> {
        let read = std::io::copy(&mut self.take(size as u64), &mut std::io::sink())?;
        if read as usize != size {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Failed to skip"));
        }
        Ok(())
    }
}
