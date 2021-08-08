use crate::lib::{handle_config, split_ipv4_string};
use crate::{
    compute_sha1, compute_sha1_with_str, make_header, Error, MyResult, Type, HEADER_PREFIX,
};
use bczhc_lib::fs::ForeachDir;
use bczhc_lib::io::ReadAll;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use clap::ArgMatches;
use std::borrow::Borrow;
use std::env::args;
use std::fs::{DirEntry, File, Permissions};
use std::io::{stdin, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn run(matches: &ArgMatches) -> MyResult<()> {
    // send:
    // transfer send [-v] [-f <file>...]

    // TODO
    let verbose = matches.is_present("verbose");

    let files = matches.values_of("file");

    let config = handle_config();

    let ipv4 = split_ipv4_string(&config.destination_ip);
    if let None = ipv4 {
        return Err(Error::InvalidIpv4);
    }
    let ipv4 = ipv4.unwrap();

    let mut tcp_stream = TcpStream::connect(SocketAddrV4::new(
        Ipv4Addr::new(ipv4.0, ipv4.1, ipv4.2, ipv4.3),
        config.port,
    ))?;

    if let Some(files) = files {
        for file_path in files {
            let path = Path::new(file_path);
            if path.is_file() {
                let result = handle_path_file(path, &mut tcp_stream);
                if let Err(e) = result {
                    eprintln!("{:?}", e);
                }
            } else if path.is_dir() {
                let result = handle_path_dir(path, &mut tcp_stream);
                if let Err(e) = result {
                    eprintln!("{:?}", e);
                }
            }
        }
    } else {
        send_stdin(&mut tcp_stream, &mut stdin());
    }

    send_end(&mut tcp_stream)?;

    Ok(())
}

fn handle_path_dir(path: &Path, tcp_stream: &mut TcpStream) -> MyResult<()> {
    let abs_path = path.canonicalize()?;
    let result = abs_path.to_str();
    if let None = result {
        return Err(Error::InvalidUTF8);
    }
    let prefix = result.unwrap();

    let ptr = tcp_stream as *mut TcpStream as usize;
    path.traversal_dir(|d| {
        match d {
            Ok(d) => {
                let r = unsafe { &mut *(ptr as *mut TcpStream) };
                let result = handle_file_in_dir(r, prefix, d);
                if let Err(e) = result {
                    eprintln!("{:?}", e);
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    })
    .unwrap();
    Ok(())
}

fn handle_file_in_dir(tcp_stream: &mut TcpStream, prefix: &str, d: &DirEntry) -> MyResult<()> {
    let abs_path = d.path().canonicalize()?;
    let cloned = abs_path.clone();
    let path_diff = abs_path.strip_prefix(prefix).unwrap();

    let mut file = File::open(cloned)?;

    let result = path_diff.to_str();
    if let None = result {
        return Err(Error::InvalidUTF8);
    }
    let path_diff = result.unwrap();

    send_file(tcp_stream, &mut file, path_diff);

    Ok(())
}

fn handle_path_file(file_path: &Path, tcp_stream: &mut TcpStream) -> MyResult<()> {
    let result = file_path.to_str();
    if let None = result {
        return Err(Error::InvalidUTF8);
    }
    let path = result.unwrap();

    let mut file = File::open(file_path)?;
    send_file(tcp_stream, &mut file, path);
    Ok(())
}

fn send_file<R>(connection: &mut TcpStream, input: &mut R, path: &str)
where
    R: Read,
{
    println!("Read...");
    let data = input.read_all();
    println!("Digest...");
    let sha1 = compute_sha1_with_str(&data, path);

    println!("Send...");
    // Header
    connection.write_all(&make_header(Type::File)).unwrap();
    // PathLength
    connection
        .write_u32::<BigEndian>(path.len() as u32)
        .unwrap();
    // Path
    connection.write_all(path.as_bytes()).unwrap();
    // ContentLength
    connection
        .write_u32::<BigEndian>(data.len() as u32)
        .unwrap();
    // Digest
    connection.write_all(&sha1).unwrap();
    // Content
    connection.write_all(&data);

    connection.flush().unwrap();
}

fn send_dir(connection: &mut TcpStream, path: &str) {
    // Header
    connection.write_all(&make_header(Type::Directory)).unwrap();
    // PathLength
    connection
        .write_u32::<BigEndian>(path.len() as u32)
        .unwrap();
    // Path
    connection.write_all(path.as_bytes()).unwrap();

    connection.flush().unwrap();
}

fn send_stdin<R>(connection: &mut TcpStream, input: &mut R)
where
    R: Read,
{
    let data = input.read_all();
    let digest = compute_sha1(&data);

    // Header
    connection.write_all(&make_header(Type::Stdin)).unwrap();
    // ContentLength
    connection
        .write_u32::<BigEndian>(data.len() as u32)
        .unwrap();
    // Digest
    connection.write_all(&digest).unwrap();
    // Content
    connection.write_all(&data).unwrap();

    connection.flush().unwrap();
}

fn send_end(connection: &mut TcpStream) -> MyResult<()> {
    connection.write_all(&make_header(Type::End))?;
    Ok(())
}
