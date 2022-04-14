use crate::attach_tcp_stream_to_stdio;
use crate::errors::*;
use std::net::TcpStream;

pub fn connect(addr: &str) -> Result<()> {
    let stream = TcpStream::connect(addr)?;

    attach_tcp_stream_to_stdio(&stream)?;

    Ok(())
}
