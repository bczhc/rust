use crate::errors::*;
use crate::{Mark, HEADER};
use bczhc_lib::fs::new_unique_file;
use bczhc_lib::io::ReadText;
use bczhc_lib::{rw_read, rw_write};
use byteorder::{BigEndian, ReadBytesExt};
use cfg_if::cfg_if;
use clap::ArgMatches;
use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::RwLock;
use tar::Archive;

static ARGUMENTS: Lazy<RwLock<Option<Arguments>>> = Lazy::new(|| RwLock::new(None));

pub fn main(matches: &ArgMatches) -> Result<()> {
    let output_dir = matches.value_of("output-dir").unwrap();

    rw_write!(ARGUMENTS).replace(Arguments {
        output_dir: String::from(output_dir),
    });

    let port = matches
        .value_of("port")
        .unwrap()
        .parse::<u16>()
        .map_err(|_| Error::InvalidPort)?;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

    println!("Listening on port {}", port);

    let accept = listener.accept()?;
    println!("Accepted connection from {}", accept.1);

    let mut stream = accept.0;
    stream.check_header()?;
    let mark = stream.read_mark()?;

    match mark {
        Mark::File => receive_file(&mut stream),
        Mark::Text => receive_text(&mut stream),
        Mark::Tar => receive_files(&mut stream),
    }
}

fn receive_file<R>(stream: &mut R) -> Result<()>
where
    R: Read,
{
    let guard = rw_read!(ARGUMENTS);
    let output_dir = &guard.as_ref().unwrap().output_dir;

    let filename_length = stream.read_u32::<BigEndian>()? as usize;
    let mut filename_bytes = vec![0_u8; filename_length];
    stream.read_exact(&mut filename_bytes)?;

    #[allow(clippy::needless_late_init)]
    let filename;
    cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::prelude::OsStrExt;
            filename = OsStr::from_bytes(&filename_bytes);
        } else {
            filename =
                OsStr::new(std::str::from_utf8(&filename_bytes).map_err(|_| Error::InvalidCharset)?);
        }
    }

    let path = PathBuf::from(output_dir).join(filename);
    let path = new_unique_file(path)?;

    let mut file = File::options().write(true).truncate(true).open(path)?;

    std::io::copy(stream, &mut file)?;

    Ok(())
}

fn receive_files<R>(stream: &mut R) -> Result<()>
where
    R: Read,
{
    let guard = rw_read!(ARGUMENTS);
    let output_dir = &guard.as_ref().unwrap().output_dir;

    let mut archive = Archive::new(stream);

    archive.unpack(output_dir)?;

    Ok(())
}

fn receive_text<R>(stream: &mut R) -> Result<()>
where
    R: Read,
{
    let text = stream.read_text()?;
    println!("Received text:");
    println!("{}", text);
    Ok(())
}

trait Ext
where
    Self: Read,
{
    fn check_header(&mut self) -> Result<()> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        if &buf != HEADER {
            return Err(Error::InvalidHeader);
        }
        Ok(())
    }

    fn read_mark(&mut self) -> Result<Mark> {
        let mark = self.read_u8()?;
        num_traits::FromPrimitive::from_u8(mark).map_or_else(|| Err(Error::InvalidMark(mark)), Ok)
    }
}

struct Arguments {
    output_dir: String,
}

impl<R: Read> Ext for R {}
