use byteorder::{BigEndian, WriteBytesExt};
use clap::{Arg, Command};
use phone_transfer::errors::*;
use phone_transfer::{Mark, HEADER};
use std::fs::File;
use std::io;
use std::io::{stdin, Write};
use std::net::{SocketAddrV4, TcpStream};
use std::path::Path;
use tar::Builder;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let matches = Command::new("phone-transfer")
        .arg(Arg::new("address").required(true))
        .arg(
            Arg::new("file")
                .required_unless_present("text")
                .help("File to transfer when in file or tar mode"),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .conflicts_with("file"),
        )
        .get_matches();

    let address = matches
        .value_of("address")
        .unwrap()
        .parse::<SocketAddrV4>()?;

    let mut stream = TcpStream::connect(address)?;
    stream.write_all(HEADER)?;

    let text_mode = matches.is_present("text");
    if text_mode {
        stream.write_u8(Mark::Text as u8)?;
        let mut stdin = stdin().lock();
        io::copy(&mut stdin, &mut stream)?;
        return Ok(());
    }
    let file = matches.value_of("file").unwrap();

    let path = Path::new(file);
    match path {
        _ if path.is_file() => {
            stream.write_all(&[Mark::File as u8])?;

            let mut file = File::open(path)?;
            let file_name = path.file_name().unwrap().to_str();
            if file_name.is_none() {
                return Err(Error::InvalidCharset);
            }

            let file_name_bytes = file_name.unwrap().as_bytes();

            stream.write_u32::<BigEndian>(file_name_bytes.len() as u32)?;
            stream.write_all(file_name_bytes)?;

            io::copy(&mut file, &mut stream)?;
        }
        _ if path.is_dir() => {
            stream.write_u8(Mark::Tar as u8)?;

            let mut builder = Builder::new(&mut stream);
            let files = WalkDir::new(path)
                .into_iter()
                .map(|x| x.unwrap())
                .filter(|x| x.path().is_file());
            for entry in files {
                let relative_path = pathdiff::diff_paths(entry.path(), path).unwrap();

                builder.append_file(&relative_path, &mut File::open(entry.path())?)?;
                println!("{:?}", entry.path());
            }
            builder.finish()?;
        }
        _ => {
            return Err(Error::UnsupportedFileType);
        }
    }

    Ok(())
}
