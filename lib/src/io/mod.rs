use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread::{spawn, JoinHandle};

use cfg_if::cfg_if;

use crate::utf8::encode_utf8;

pub mod errors;

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
    /// The std lib implementation: [`Stdin::lines`]
    ///
    /// # Examples
    /// ```no_run
    /// use bczhc_lib::io::ReadLines;
    /// use std::io::stdin;
    ///
    /// let mut stdin = stdin();
    /// let lines = ReadLines::lines(&mut stdin);
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
                    return if read.is_empty() {
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
        Some(String::from_utf8(read).unwrap())
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
    for b in bytes.iter().take(size) {
        put_c_char(*b)?;
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

pub fn pipe_thread<R, W>(reader: R, writer: W) -> JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    fn pipe<R, W>(mut reader: R, mut writer: W) -> io::Result<()>
    where
        R: Read,
        W: Write,
    {
        let mut buf = [0_u8; 4096];
        loop {
            let size = reader.read(&mut buf)?;
            if size == 0 {
                break;
            }
            writer.write_all(&buf[..size])?;
        }
        Ok(())
    }

    spawn(move || pipe(reader, writer))
}

pub fn attach_tcp_stream_to_stdio(stream: &mut TcpStream) -> io::Result<()> {
    cfg_if! {
        if #[cfg(unix)] {
            unix::attach_stream_to_stdio(stream)
        } else {
            generic::attach_tcp_stream_to_stdio(stream)
        }
    }
}

pub fn interact_two_stream(stream1: &mut TcpStream, stream2: &mut TcpStream) -> io::Result<()> {
    cfg_if! {
        if #[cfg(unix)] {
            unix::interact_two_stream(stream1, stream2)
        } else {
            generic::interact_two_stream(stream1, stream2)
        }
    }
}

#[cfg(unix)]
pub mod unix {
    use std::io;
    use std::io::{stdin, stdout, Read, Write};
    use std::net::TcpStream;
    use std::os::unix::io::AsRawFd;

    use polling::{Event, Poller};

    macro_rules! interact_two_stream_code_gen {
        ($stream1:expr, $stream2:expr,
        $stream1_dest:expr, $stream2_dest:expr) => {
            let stream1_key = 0;
            let stream2_key = 1;

            let poller = Poller::new()?;
            poller.add(&*$stream1, Event::readable(stream1_key))?;
            poller.add(&*$stream2, Event::readable(stream2_key))?;

            let mut events = Vec::new();
            let mut buf = [0_u8; 4096];
            'poll_loop: loop {
                events.clear();
                poller.wait(&mut events, None)?;
                for ev in &events {
                    let key = ev.key;
                    let readable = ev.readable;
                    match ev.key {
                        _ if key == stream1_key && readable => {
                            let size = $stream1.read(&mut buf)?;
                            if size == 0 {
                                break 'poll_loop;
                            }
                            $stream1_dest.write_all(&buf[..size])?;
                        }
                        _ if key == stream2_key && readable => {
                            let size = $stream2.read(&mut buf)?;
                            if size == 0 {
                                break 'poll_loop;
                            }
                            $stream2_dest.write_all(&buf[..size])?;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                    poller.modify(&*$stream1, Event::readable(stream1_key))?;
                    poller.modify(&*$stream2, Event::readable(stream2_key))?;
                }
            }

            return Ok(())
        };
    }

    pub fn attach_stream_to_stdio<S>(stream: &mut S) -> io::Result<()>
    where
        S: Read + Write + AsRawFd,
    {
        let stdin = &mut stdin().lock();
        let stdout = &mut stdout().lock();

        interact_two_stream_code_gen!(stream, stdin, stdout, stream);
    }

    pub fn interact_two_stream(stream1: &mut TcpStream, stream2: &mut TcpStream) -> io::Result<()> {
        interact_two_stream_code_gen!(stream1, stream2, stream2, stream1);
    }
}

#[cfg(not(unix))]
pub mod generic {
    use std::io;
    use std::io::{stdin, stdout};
    use std::net::TcpStream;

    use crate::io::pipe_thread;

    pub fn attach_tcp_stream_to_stdio(stream: &mut TcpStream) -> io::Result<()> {
        let t1 = pipe_thread(stdin(), stream.try_clone()?);
        let t2 = pipe_thread(stream.try_clone()?, stdout());
        t1.join().unwrap()?;
        t2.join().unwrap()?;
        Ok(())
    }

    pub fn interact_two_stream(stream1: &mut TcpStream, stream2: &mut TcpStream) -> io::Result<()> {
        let t1 = pipe_thread(stream1.try_clone()?, stream2.try_clone()?);
        let t2 = pipe_thread(stream2.try_clone()?, stream1.try_clone()?);
        t1.join().unwrap()?;
        t2.join().unwrap()?;
        Ok(())
    }
}
