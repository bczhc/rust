use std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::RwLock;
use std::thread::spawn;

use clap::{Arg, Command};
use once_cell::sync::Lazy;

use bczhc_lib::{rw_read, rw_write};
use forward::errors::*;

static ARGUMENTS: Lazy<RwLock<Arguments>> = Lazy::new(|| RwLock::new(Arguments::default()));

fn main() -> Result<()> {
    let matches = Command::new("forward")
        .arg(Arg::new("address").required(true))
        .arg(Arg::new("forward-port").required(true))
        .get_matches();

    let address = matches.value_of("address").unwrap();
    let forward_port: u16 = matches.value_of("forward-port").unwrap().parse()?;

    let to_addr = address.to_socket_addrs()?.next().unwrap();
    rw_write!(ARGUMENTS).to_addr = Some(to_addr);

    let listener = TcpListener::bind(SocketAddrV4::new("0.0.0.0".parse().unwrap(), forward_port))?;

    loop {
        let accept = listener.accept()?;
        let client_addr = accept.1;
        println!("Accept: {:?}", client_addr);
        spawn(move || {
            handle_connection(accept.0).unwrap();
            println!("Disconnected: {:?}", client_addr);
        });
    }
}

fn handle_connection(access_stream: TcpStream) -> Result<()> {
    let to_addr = rw_read!(ARGUMENTS).to_addr.unwrap();
    let local_stream = TcpStream::connect(to_addr)?;

    bczhc_lib::io::poll::poll_between_two_streams(&access_stream, &local_stream)?;
    Ok(())
}

#[derive(Default)]
struct Arguments {
    to_addr: Option<SocketAddr>,
}
