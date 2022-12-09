use crate::errors::*;
use crate::{crypto_interact_two_streams, KEY, KEY_REF};
use bczhc_lib::io::interact_two_stream;
use bczhc_lib::mutex_lock;
use clap::ArgMatches;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::spawn;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let server_addr = matches.get_one::<String>("server").unwrap().clone();
    let port = *matches.get_one::<u16>("port").unwrap();

    let server_addr = Arc::new(server_addr);

    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port))?;
    loop {
        let accepted = listener.accept()?;
        println!("Accepted: {}", accepted.1);
        let server_addr = Arc::clone(&server_addr);
        spawn(move || handle_connection(accepted.0, &server_addr).unwrap());
    }
}

pub fn handle_connection(mut client: TcpStream, server_addr: &str) -> Result<()> {
    let mut server = TcpStream::connect(server_addr)?;

    crypto_interact_two_streams(&mut client, &mut server)?;

    Ok(())
}
