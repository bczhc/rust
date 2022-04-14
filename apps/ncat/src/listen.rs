use std::net::{SocketAddr, TcpListener};

use crate::errors::*;
use crate::poll::attach_tcp_stream_to_stdio;

pub fn listen(port: u16) -> Result<()> {
    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
    let listener = TcpListener::bind(addr)?;

    let accept = listener.accept()?;
    println!("Accepted: {:?}", accept.1);

    let stream = accept.0;
    attach_tcp_stream_to_stdio(&stream)?;

    Ok(())
}
