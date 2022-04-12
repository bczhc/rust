use bczhc_lib::{rw_read, rw_write};
use clap::{Arg, Command};
use forward::errors::*;
use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread::spawn;

static ARGUMENTS: Lazy<RwLock<Arguments>> = Lazy::new(|| RwLock::new(Arguments::default()));

fn main() -> Result<()> {
    let matches = Command::new("forward")
        .arg(Arg::new("address").required(true))
        .arg(Arg::new("forward-port").required(true))
        .get_matches();

    let address = matches.value_of("address").unwrap();
    let forward_port: u16 = matches.value_of("forward-port").unwrap().parse()?;

    rw_write!(ARGUMENTS).address = Some(String::from(address));

    let listener = TcpListener::bind(SocketAddrV4::new("0.0.0.0".parse().unwrap(), forward_port))?;

    loop {
        let accept = listener.accept()?;
        println!("Accept: {:?}", accept);
        spawn(move || {
            handle_connection(accept.0).unwrap();
        });
    }
}

fn handle_connection(access_stream: TcpStream) -> Result<()> {
    let guard = rw_read!(ARGUMENTS);
    let address = guard.address.as_ref().unwrap();

    let local_stream = TcpStream::connect(address)?;

    let access_stream_arc = Arc::new(access_stream);
    let local_stream_arc = Arc::new(local_stream);

    let access_stream = Arc::clone(&access_stream_arc);
    let local_stream = Arc::clone(&local_stream_arc);
    let t1 = spawn(move || {
        // copy access_stream to local_stream
        let mut buf = [0; 1024];
        loop {
            let n = access_stream.as_ref().read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            println!("access_stream -> local_stream: {:?}", &buf[..n]);
            local_stream.as_ref().write_all(&buf[..n]).unwrap();
        }
    });

    let access_stream = Arc::clone(&access_stream_arc);
    let local_stream = Arc::clone(&local_stream_arc);
    let t2 = spawn(move || {
        // copy local_stream to access_stream
        let mut buf = [0; 1024];
        loop {
            let n = local_stream.as_ref().read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            println!("local_stream -> access_stream: {:?}", &buf[..n]);
            access_stream.as_ref().write_all(&buf[..n]).unwrap();
        }
    });

    t1.join()?;
    t2.join()?;
    Ok(())
}

#[derive(Default)]
struct Arguments {
    address: Option<String>,
}
