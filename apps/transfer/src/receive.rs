use crate::lib::handle_config;
use crate::{compute_sha1, compute_sha1_with_path, read_header, Error, MyResult, Type};
use bczhc_lib::io::{put_char, OpenOrCreate, ReadAll};
use byteorder::{BigEndian, ReadBytesExt};
use clap::ArgMatches;
use std::fs::{create_dir, File};
use std::io::{BufReader, BufWriter, Cursor, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::Path;

pub fn run(matches: &ArgMatches) -> MyResult<()> {
    // receive:
    // transfer receive [-v]

    let verbose = matches.is_present("verbose");

    let config = handle_config();
    println!("Configuration: {:?}", config);

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))?;

    let (mut tcp_stream, socket_addr) = listener.accept()?;
    println!("Accept: {:?}", socket_addr);
    loop {
        let header = read_header(&mut tcp_stream)?;

        let file_type = header.file_type;

        match file_type {
            Type::File => {
                receive_file(&mut tcp_stream);
            }
            Type::Directory => {
                receive_dir(&mut tcp_stream);
            }
            Type::Stdin => {
                receive_stdin(&mut tcp_stream);
            }
            Type::End => {
                break;
            }
        }
    }

    Ok(())
}

fn receive_file(stream: &mut TcpStream) -> MyResult<()> {
    let path = read_path(stream)?;
    // ContentLength
    let content_len = stream.read_u32::<BigEndian>()? as usize;
    // Digest
    let digest = read_digest(stream)?;
    // Content
    let mut data = vec![0_u8; content_len];
    stream.read_exact(&mut data)?;

    let sha1 = compute_sha1_with_path(&data, &path);
    if sha1 != digest {
        return Err(Error::DigestCheckError);
    }

    let mut file = File::open_or_create(&path)?;
    file.write_all(&data)?;
    println!("{}", path);

    Ok(())
}

fn receive_dir(stream: &mut TcpStream) -> MyResult<()> {
    let path = read_path(stream)?;
    let result = create_dir(&path);
    if let Err(e) = result {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(Error::IOError(e));
        }
    }
    Ok(())
}

fn receive_stdin(stream: &mut TcpStream) -> MyResult<()> {
    let content_len = stream.read_u32::<BigEndian>()? as usize;
    let digest = read_digest(stream)?;
    let mut data = vec![0_u8; content_len];
    stream.read_exact(&mut data)?;

    let sha1 = compute_sha1(&data);
    if digest != sha1 {
        return Err(Error::DigestCheckError);
    }

    for c in data {
        put_char(c);
    }

    Ok(())
}

fn read_path(stream: &mut TcpStream) -> MyResult<String> {
    // PathLength
    let path_len = stream.read_u32::<BigEndian>()? as usize;
    // Path
    let mut path_buf = vec![0_u8; path_len];
    stream.read_exact(&mut path_buf)?;
    let path = String::from_utf8(path_buf)?;
    Ok(path)
}

#[inline]
fn read_digest(stream: &mut TcpStream) -> MyResult<[u8; 20]> {
    let mut buf = [0_u8; 20];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}
