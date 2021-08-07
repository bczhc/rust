use crate::compute_sha1;
use crate::lib::handle_config;
use byteorder::{BigEndian, ReadBytesExt};
use clap::ArgMatches;
use lib::io::{OpenOrCreate, ReadAll};
use std::fs::File;
use std::io::{Cursor, Error, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

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
    let mut header_buf = [0_u8; 5];
    let result = tcp_stream.read_exact(&mut header_buf);
    if let Err(e) = check_eof_err(&result) {
        return Err(e.to_string());
    }

    let mut sha1_buf = [0_u8; 20];
    let result = tcp_stream.read_exact(&mut sha1_buf);
    if let Err(e) = check_eof_err(&result) {
        return Err(e.to_string());
    }

    let mut length_buf = [0_u8; 4];
    let result = tcp_stream.read_exact(&mut length_buf);
    if let Err(e) = check_eof_err(&result) {
        return Err(e.to_string());
    }

    let mut cursor = Cursor::new(length_buf);
    let content_length = cursor.read_u32::<BigEndian>().unwrap() as usize;

    let mut data = vec![0_u8; content_length];

    let result = tcp_stream.read_exact(&mut data);
    if let Err(e) = check_eof_err(&result) {
        return Err(e.to_string());
    }

    let sha1 = compute_sha1(&data);
    if sha1_buf != sha1 {
        return Err(String::from("Integrity check failed"));
    }

    let mut output_file = File::open_or_create("output").unwrap();
    output_file.write_all(&data);
    output_file.flush();

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
