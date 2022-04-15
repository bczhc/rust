use std::net::TcpStream;

use bczhc_lib::io::attach_tcp_stream_to_stdio;

use crate::errors::*;

pub fn connect(addr: &str) -> Result<()> {
    let mut stream = TcpStream::connect(addr)?;

    attach_tcp_stream_to_stdio(&mut stream)?;

    Ok(())
}
