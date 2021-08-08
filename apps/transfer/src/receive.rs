use crate::lib::handle_config;
use crate::{compute_sha1, Type};
use byteorder::{BigEndian, ReadBytesExt};
use clap::ArgMatches;
use lib::io::{put_char, OpenOrCreate, ReadAll};
use std::fs::{create_dir, File};
use std::io::{Cursor, Error, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::Path;

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    // receive:
    // transfer receive [-v]

    let verbose = matches.is_present("verbose");

    let config = handle_config();
    println!("Configuration: {:?}", config);

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))
        .expect("Failed to bind TCP listener");

    let (mut tcp_stream, socket_addr) = listener.accept().expect("Accept error");
    println!("Accept: {:?}", socket_addr);
    loop {
        let mut header_buf = [0_u8; 5];
        let result = tcp_stream.read_exact(&mut header_buf);
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }

        let result = tcp_stream.read_u8();
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }
        let end = result.unwrap() != 0;
        if end {
            break;
        }

        let result = tcp_stream.read_u32::<BigEndian>();
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }
        let path_len = result.unwrap() as usize;

        let mut path_buf = vec![0_u8; path_len];
        let result = tcp_stream.read_exact(&mut path_buf);
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }
        let file_path = String::from_utf8(path_buf);
        if let Err(e) = file_path {
            return Err(e.to_string());
        }
        let file_path = file_path.unwrap();

        let result = tcp_stream.read_u8();
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }
        let file_type = Type::value_of(result.unwrap());
        if let None = file_type {
            return Err(String::from("Invalid file type code"));
        }
        let file_type = file_type.unwrap();

        let result = tcp_stream.read_u32::<BigEndian>();
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }
        let content_len = result.unwrap() as usize;

        let mut sha1_buf = [0_u8; 20];
        let result = tcp_stream.read_exact(&mut sha1_buf);
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }

        let mut data = vec![0_u8; content_len];
        let result = tcp_stream.read_exact(&mut data);
        if let Err(e) = check_eof_err(&result) {
            return Err(e.to_string());
        }

        let sha1 = compute_sha1(&data, file_path.as_str());
        if sha1_buf != sha1 {
            return Err(String::from("Integrity check failed"));
        }

        match file_type {
            Type::File => {
                let mut output_file = File::open_or_create(file_path.as_str()).unwrap();
                output_file.write_all(&data).unwrap();
                output_file.flush().unwrap();
            }
            Type::Directory => {
                create_dir(Path::new(file_path.as_str())).unwrap();
            }
            Type::Stdin => {
                for c in data {
                    put_char(c);
                }
            }
        }
    }

    Ok(())
}

fn check_eof_err<R>(result: &std::io::Result<R>) -> Result<(), &Error> {
    if let Err(e) = result {
        if let ErrorKind::UnexpectedEof = e.kind() {
            return Err(e);
        } else {
            panic!("IO error: {:?}", e);
        }
    }
    Ok(())
}
