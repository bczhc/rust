use crate::crypto_interact_two_streams;
use crate::errors::*;
use bczhc_lib::io::interact_two_stream;
use bczhc_lib::mutex_lock;
use clap::ArgMatches;
use once_cell::sync::Lazy;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let address = matches.get_one::<String>("address").unwrap().clone();
    let port = *matches.get_one::<u16>("port").unwrap();

    let address = Arc::new(address);

    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port))?;
    loop {
        let client = listener.accept()?;
        println!("Accepted: {}", client.1);
        let address = Arc::clone(&address);
        spawn(move || handle_connection(client.0, &address).unwrap());
    }
}

fn handle_connection(mut client: TcpStream, target_addr: &str) -> Result<()> {
    let mut target = TcpStream::connect(target_addr)?;

    crypto_interact_two_streams(&mut target, &mut client)?;

    Ok(())
}
