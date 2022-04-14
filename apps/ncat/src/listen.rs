use crate::attach_tcp_stream_to_stdio;
use crate::errors::*;
use std::net::{SocketAddr, TcpListener};

pub fn listen(port: u16) -> Result<()> {
    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
    let listener = TcpListener::bind(addr)?;

    let accept = listener.accept()?;
    println!("Accepted: {:?}", accept.1);

    let stream = accept.0;
    attach_tcp_stream_to_stdio(&stream)?;

    Ok(())
}
