use crate::errors::*;
use crate::{Mark, HEADER};
use bczhc_lib::io::ReadText;
use bczhc_lib::time::current_time_millis;
use byteorder::{BigEndian, WriteBytesExt};
use cfg_if::cfg_if;
use clap::ArgMatches;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{stdin, Cursor, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use walkdir::WalkDir;

enum PipeSendMode {
    File,
    Text,
}

enum SendMode<'a> {
    Pipe(PipeSendMode),
    Path(&'a str),
}

pub fn main(matches: &ArgMatches) -> Result<()> {
    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();

    let pipe_mode = if matches.is_present("pipe-mode") {
        if matches.is_present("file-mode") {
            SendMode::Pipe(PipeSendMode::File)
        } else if matches.is_present("text-mode") {
            SendMode::Pipe(PipeSendMode::Text)
        } else {
            unreachable!();
        }
    } else {
        SendMode::Path(matches.value_of("file").unwrap())
    };

    let mut stream = TcpStream::connect(format!("{}:{}", ip, port))?;

    match pipe_mode {
        SendMode::Pipe(mode) => {
            let mut stdin = stdin().lock();
            match mode {
                PipeSendMode::File => {
                    send_file(&mut stream, &mut stdin, current_time_millis().to_string())?;
                }
                PipeSendMode::Text => {
                    send_text(&mut stream, &stdin.read_text()?)?;
                }
            }
        }
        SendMode::Path(path) => {
            let path = Path::new(path);
            if path.is_dir() {
                send_dir(&mut stream, path)?;
            } else {
                let filename = path
                    .file_name()
                    .map_or_else(|| Err(Error::UnknownFilename), Ok)?;
                send_file(&mut stream, &mut File::open(path)?, filename)?;
            }
        }
    }
    Ok(())
}

pub fn send_file<S, R, N>(stream: &mut S, reader: &mut R, filename: N) -> Result<()>
where
    S: Write,
    R: Read,
    N: AsRef<OsStr>,
{
    stream.write_header()?;
    stream.write_mark(Mark::File)?;

    let filename = filename.as_ref();
    #[allow(clippy::needless_late_init)]
    let filename_bytes;
    cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::prelude::OsStrExt;
            filename_bytes = filename.as_bytes();
        } else {
            let filename: &str = filename
                .to_str()
                .map_or_else(|| Err(Error::InvalidCharset), Ok)?;
            filename_bytes = filename.as_bytes();
        }
    }

    stream.write_u32::<BigEndian>(filename_bytes.len() as u32)?;
    stream.write_all(filename_bytes)?;

    std::io::copy(reader, stream)?;
    Ok(())
}

pub fn send_dir<S, P>(stream: &mut S, dir: P) -> Result<()>
where
    S: Write,
    P: AsRef<Path>,
{
    stream.write_header()?;
    stream.write_mark(Mark::Tar)?;

    use tar::Builder;
    let mut builder = Builder::new(stream);
    let files = WalkDir::new(&dir)
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| x.path().is_file());
    for entry in files {
        let relative_path = pathdiff::diff_paths(entry.path(), &dir).unwrap();

        builder.append_file(&relative_path, &mut File::open(entry.path())?)?;
        println!("{:?}", entry.path());
    }
    builder.finish()?;
    Ok(())
}

pub fn send_text<S>(stream: &mut S, text: &str) -> Result<()>
where
    S: Write,
{
    stream.write_header()?;
    stream.write_mark(Mark::Text)?;

    let mut cursor = Cursor::new(text);
    io::copy(&mut cursor, stream)?;
    Ok(())
}

trait Ext<W>
where
    W: Write,
    Self: Write,
{
    fn write_header(&mut self) -> io::Result<()> {
        self.write_all(HEADER)
    }

    fn write_mark(&mut self, mark: Mark) -> io::Result<()> {
        self.write_u8(mark as u8)
    }
}

impl<W: Write> Ext<W> for W {}
