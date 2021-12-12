use crate::utf8::encode_utf8;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Cursor, Error, ErrorKind, Read};
use std::path::Path;
use std::sync::mpsc::RecvError;

trait ReadLine {
    /// Read lines without the end newline mark (CR and/or LF)
    fn read_line_without_line_terminator(&mut self) -> Option<String>;
}

pub struct Lines<'a, T>
where
    T: Read,
{
    readable: &'a mut T,
}

pub trait ReadLines<T>
where
    T: Read,
{
    /// Read lines from the readable stream
    ///
    /// # Examples
    /// ```no_run
    /// use bczhc_lib::io::ReadLines;
    /// use std::io::stdin;
    ///
    /// let mut stdin = stdin();
    /// let lines = stdin.lines();
    /// for line in lines {
    ///     println!("{}", line);
    /// }
    /// ```
    fn lines(&mut self) -> Lines<T>;
}

impl<T> Iterator for Lines<'_, T>
where
    T: Read,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        super::io::ReadLine::read_line_without_line_terminator(self.readable)
    }
}

impl<T> ReadLines<T> for T
where
    T: Read,
{
    fn lines(&mut self) -> Lines<T> {
        Lines { readable: self }
    }
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
                    return if read.len() == 0 {
                        None
                    } else {
                        Some(String::from_utf8(read).unwrap())
                    };
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

/// Write a byte to [`stdout`] immediately
/// # Examples
///
/// ```no_run
/// use bczhc_lib::io::put_c_char;
///
/// put_c_char(b'a').unwrap();
/// ```
///
/// # Errors
/// When the C `putchar` returns [`libc::EOF`]
///
#[inline]
pub fn put_c_char(c: u8) -> std::io::Result<()> {
    unsafe {
        let r = libc::write(1, &c as *const u8 as *const libc::c_void, 1);
        if r != 1 {
            // TODO: get error kind from `errno`
            return Err(std::io::Error::from(ErrorKind::Other));
        }
    }
    Ok(())
}

/// Write a rust [`char`] to [`stdout`] immediately
///
/// # Examples
///
/// ```no_run
/// use bczhc_lib::io::put_char;
///
/// put_char('ö').unwrap();
/// ```
#[inline]
pub fn put_char(c: char) -> std::io::Result<()> {
    let mut bytes = [0_u8; 4];
    let size = encode_utf8(c as u32, &mut bytes);
    for i in 0..size {
        put_c_char(bytes[i])?;
    }
    Ok(())
}

pub trait TryReadExact {
    /// Read exact data
    ///
    /// This function blocks. It reads exact data, so the return value is the buffer's when
    /// it doesn't reach EOF.
    ///
    /// If it reaches EOF, the return value is the bytes it has read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::stdin;
    /// use bczhc_lib::io::TryReadExact;
    ///
    /// let mut stdin = stdin();
    /// let mut buf = [0_u8; 5];
    /// loop {
    ///     let result = stdin.try_read_exact(&mut buf);
    ///     match result {
    ///         Ok(r) => {
    ///             if r == 0 {
    ///                 break;
    ///             }
    ///             println!("Read: {:?}", &buf[..r]);
    ///         }
    ///         Err(e) => {
    ///             eprintln!("IO error: {}", e);
    ///         }
    ///     }
    /// }
    /// ```
    fn try_read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

impl<R> TryReadExact for R
where
    R: Read,
{
    fn try_read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0_usize;
        loop {
            let result = self.read(&mut buf[read..]);
            match result {
                Ok(r) => {
                    if r == 0 {
                        return Ok(read);
                    }
                    read += r;
                    if read == buf.len() {
                        return Ok(read);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
