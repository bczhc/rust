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
        .arg(Arg::new("dest-addr").required(true))
        .arg(Arg::new("listen-port").required(true))
        .get_matches();

    let dest_addr = matches.value_of("dest-addr").unwrap();
    let listen_port: u16 = matches.value_of("listen-port").unwrap().parse()?;

    let dest_addr = dest_addr.to_socket_addrs()?.next().unwrap();
    rw_write!(ARGUMENTS).dest_addr = Some(dest_addr);

    let listener = TcpListener::bind(SocketAddrV4::new("0.0.0.0".parse().unwrap(), listen_port))?;

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
    let to_addr = rw_read!(ARGUMENTS).dest_addr.unwrap();
    let local_stream = TcpStream::connect(to_addr)?;

    bczhc_lib::io::poll::poll_between_two_streams(&access_stream, &local_stream)?;
    Ok(())
}

#[derive(Default)]
struct Arguments {
    dest_addr: Option<SocketAddr>,
}
