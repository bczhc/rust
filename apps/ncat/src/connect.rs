use std::net::TcpStream;

use crate::errors::*;
use crate::poll::attach_tcp_stream_to_stdio;

pub fn connect(addr: &str) -> Result<()> {
    let stream = TcpStream::connect(addr)?;

    attach_tcp_stream_to_stdio(&stream)?;

    Ok(())
}
