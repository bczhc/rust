use crate::lib::{handle_config, split_ipv4_string};
use crate::{HEADER, compute_sha1};
use clap::ArgMatches;
use lib::io::{OpenOrCreate, ReadAll};
use sha1::{Digest, Sha1};
use std::fs::{File, Permissions};
use std::io::{stdin, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use byteorder::{WriteBytesExt, ByteOrder, BigEndian};

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    // send:
    // transfer send [-v] [-f <file>...]

    // TODO
    let verbose = matches.is_present("verbose");
    let mut file = matches.value_of("file");

    let config = handle_config();
    println!("Configuration: {:?}\n", config);

    let ipv4 = split_ipv4_string(&config.destination_ip).expect("Invalid ipv4 format");
    let mut tcp_conn = TcpStream::connect(SocketAddrV4::new(
        Ipv4Addr::new(ipv4.0, ipv4.1, ipv4.2, ipv4.3),
        config.port,
    ))
    .expect("Failed to connect");

    if let None = file {
        send(&mut tcp_conn, &mut stdin());
    } else {
        let mut file = File::open(file.unwrap()).unwrap();
        send(&mut tcp_conn, &mut file);
    }

    Ok(())
}

/// ### Structure:
///
/// | Header | Digest | ContentLength | Content |
fn send<R>(connection: &mut TcpStream, input: &mut R)
where
    R: Read,
{
    let data = input.read_all();
    let sha1 = compute_sha1(&data);

    connection.write_all(HEADER).unwrap();
    connection.write_all(&sha1).unwrap();
    connection.write_u32::<BigEndian>(data.len() as u32).unwrap();
    connection.write_all(&data);
    connection.flush().unwrap();
}
