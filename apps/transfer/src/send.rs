use crate::lib::{handle_config, split_ipv4_string};
use crate::{compute_sha1, Type, HEADER};
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use clap::ArgMatches;
use lib::fs::ForeachDir;
use lib::io::{OpenOrCreate, ReadAll};
use sha1::{Digest, Sha1};
use std::borrow::Borrow;
use std::env::args;
use std::fs::{File, Permissions};
use std::io::{stdin, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    // send:
    // transfer send [-v] [-f <file>...]

    // TODO
    let verbose = matches.is_present("verbose");

    let files = matches.values_of("file");

    let config = handle_config();
    println!("Configuration: {:?}\n", config);

    let ipv4 = split_ipv4_string(&config.destination_ip).expect("Invalid ipv4 format");
    let mut tcp_conn = TcpStream::connect(SocketAddrV4::new(
        Ipv4Addr::new(ipv4.0, ipv4.1, ipv4.2, ipv4.3),
        config.port,
    ));
    if let Err(e) = tcp_conn {
        return Err(e.to_string());
    }
    let mut tcp_conn = tcp_conn.unwrap();

    let config = handle_config();
    println!("Configuration: {:?}\n", config);

    if let Some(files) = files {
        for file_path in files {
            let path = Path::new(file_path);
            if path.is_file() {
                let result = handle_path_file(
                    path,
                    path.file_name().unwrap().to_str().unwrap(),
                    &mut tcp_conn,
                );
                if let Err(e) = result {
                    println!("{:?}", e);
                }
            } else if path.is_dir() {
                let result = handle_path_dir(path, &mut tcp_conn);
                if let Err(e) = result {
                    eprintln!("{:?}", e);
                }
            }
        }
    } else {
        send(&mut tcp_conn, &mut stdin(), None, Type::Stdin);
    }

    send_end(&mut tcp_conn);

    Ok(())
}

fn handle_path_dir(path: &Path, tcp_conn: &mut TcpStream) -> std::io::Result<()> {
    let result = path.canonicalize();
    if let Err(e) = result {
        return Err(e);
    }

    let path = result.unwrap();
    let prefix = path.to_str().unwrap();

    let ptr = tcp_conn as *mut TcpStream as usize;
    path.traversal_dir(|d| {
        let cann_path = d.unwrap().path().canonicalize();

        if let Ok(cann_path) = cann_path {
            let relative_path = cann_path.strip_prefix(prefix).unwrap();
            let r = unsafe { &mut *(ptr as *mut TcpStream) };
            let result = handle_path_file(cann_path.as_path(), relative_path.to_str().unwrap(), r);
            if let Err(e) = result {
                eprintln!("{:?}", e);
            }
        } else if let Err(e) = cann_path {
            eprintln!("{:?}", e);
        }
    })
    .unwrap();

    send_end(tcp_conn);
    Ok(())
}

fn handle_path_file(
    file_path: &Path,
    path_diff: &str,
    tcp_conn: &mut TcpStream,
) -> std::io::Result<()> {
    let file = File::open(file_path);
    if let Err(e) = file {
        return Err(e);
    }
    let result = file_path.canonicalize();
    if let Err(e) = result {
        return Err(e);
    }
    let path_cann = result.unwrap();

    let result = path_cann.parent();
    if let None = result {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Cannot found parent directory",
        ));
    }
    let parent = result.unwrap();

    let path_diff = path_cann.strip_prefix(parent).unwrap();

    send(
        tcp_conn,
        &mut file.unwrap(),
        Some(path_diff.to_str().unwrap()),
        Type::File,
    );
    Ok(())
}

/// ### Structure:
///
/// | Header (5) | End (1) | PathLength (4) | Path | Type (1) | ContentLength (4) | Digest | Content |
fn send<R>(connection: &mut TcpStream, input: &mut R, path: Option<&str>, file_type: Type)
where
    R: Read,
{
    println!("Read...");
    let data = input.read_all();
    println!("Digest...");
    let sha1 = compute_sha1(&data, path.unwrap_or(""));

    println!("Send...");
    connection.write_all(HEADER).unwrap();
    connection.write_u8(0).unwrap();
    match file_type {
        Type::File | Type::Directory => {
            let path = path.unwrap();
            connection
                .write_u32::<BigEndian>(path.len() as u32)
                .unwrap();
            connection.write_all(path.as_bytes()).unwrap();
        }
        Type::Stdin => {
            connection.write_all(&[0, 0, 0, 0]).unwrap();
        }
    };
    connection.write_u8(file_type.value()).unwrap();
    connection
        .write_u32::<BigEndian>(data.len() as u32)
        .unwrap();
    connection.write_all(&sha1).unwrap();
    connection.write_all(&data);
    connection.flush().unwrap();
}

fn send_end(connection: &mut TcpStream) {
    connection.write_all(HEADER).unwrap();
    connection.write_u8(1).unwrap();
    connection.flush().unwrap();
}
