use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Cursor, Error, ErrorKind, Read};
use std::path::Path;

pub trait ReadLine {
    /// Read lines without the end newline mark (CR and/or LF)
    ///
    /// # Examples
    /// ```no_run
    /// use std::fs::File;
    /// use bczhc_lib::io::ReadLine;
    ///
    /// let mut file = File::open("a.txt").expect("Failed to open file");
    /// loop {
    ///     let result = file.read_line_without_line_terminator();
    ///     if let Some(line) = result {
    ///         println!("{}", line);
    ///     } else {
    ///         break;
    ///     }
    /// }
    /// ```
    fn read_line_without_line_terminator(&mut self) -> Option<String>;
}

impl<T> ReadLine for T
where
    T: Read,
{
    fn read_line_without_line_terminator(&mut self) -> Option<String> {
        // TODO: not read the last line when files don't have EOL
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

    fn open_append_file<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
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

pub trait ReadAll {
    /// Read all data until the end
    ///
    /// # Examples
    /// ```no_run
    /// use std::fs::File;
    /// use bczhc_lib::io::ReadAll;
    ///
    /// let mut file = File::open("aa").unwrap();
    /// let data = file.read_all();
    /// println!("Data: {:?}", data);
    /// ```
    fn read_all(&mut self) -> Vec<u8>;
}

impl<R> ReadAll for R
where
    R: Read,
{
    fn read_all(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = [0_u8; 4096];
        loop {
            let read_len = self.read(&mut buf[..]).unwrap();
            if read_len == 0 {
                // EOF
                break;
            }
            for b in buf[..read_len].iter() {
                out.push(*b);
            }
        }
        out
    }
}

#[inline]
pub fn put_char(c: u8) -> std::io::Result<()> {
    unsafe {
        if libc::putchar(c as libc::c_int) == libc::EOF {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        };
    }
    Ok(())
}
