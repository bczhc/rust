use crate::errors::*;
use bczhc_lib::io::interact_two_stream;
use bczhc_lib::mutex_lock;
use clap::ArgMatches;
use once_cell::sync::Lazy;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Mutex;
use std::thread::spawn;

static TARGET_ADDRESS: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn main(matches: &ArgMatches) -> Result<()> {
    let address = matches.get_one::<String>("address").unwrap();
    let port = *matches.get_one::<u16>("port").unwrap();

    mutex_lock!(TARGET_ADDRESS).replace(address.clone());

    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port))?;
    loop {
        let client = listener.accept()?;
        println!("Accepted: {}", client.1);
        let client = client.0;
        spawn(move || handle_client(client).unwrap());
    }
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut target_stream = TcpStream::connect(mutex_lock!(TARGET_ADDRESS).as_ref().unwrap())?;

    Ok(())
}
