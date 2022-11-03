use crate::{
    check_option, compute_sha1, compute_sha1_with_str, parse_port_str, read_header, Configs, Error,
    MyResult, Type,
};
use bczhc_lib::io::{put_c_char, OpenOrCreate};
use byteorder::{BigEndian, ReadBytesExt};
use clap::ArgMatches;
use std::fs::{create_dir, File};
use std::io::{stdout, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

use crate::lib::{read_config_file, search_config};

pub fn run(matches: &ArgMatches) -> MyResult<()> {
    // receive:
    // transfer receive [-s]

    let _verbose = matches.is_present("verbose");
    let stream_mode = matches.is_present("stream-mode");

    let config = read_config_file()?;
    let result = search_config(&config, Configs::Port.key());
    let port = check_option(result, Error::NoConfig(Configs::Port))?;
    let port = parse_port_str(port)?;

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))?;
    if !stream_mode {
        println!("Listening at port {}", port);
    }

    let (mut tcp_stream, _) = listener.accept()?;
    if stream_mode {
        return receive_stream_mode(&mut tcp_stream);
    }

    loop {
        let header = read_header(&mut tcp_stream)?;

        let file_type = header.file_type;

        match file_type {
            Type::File => {
                receive_file(&mut tcp_stream)?;
            }
            Type::Directory => {
                receive_dir(&mut tcp_stream)?;
            }
            Type::Stdin => {
                receive_stdin(&mut tcp_stream)?;
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

    let sha1 = compute_sha1_with_str(&data, &path);
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
    let result = create_dir(path);
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
        put_c_char(c)?;
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

fn receive_stream_mode(stream: &mut TcpStream) -> MyResult<()> {
    std::io::copy(stream, &mut stdout())?;
    Ok(())
}
