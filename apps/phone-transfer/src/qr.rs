use crate::errors::*;
use pnet::datalink::NetworkInterface;
use std::io::{stderr, stdin, Write};

pub fn print_addr_qr(port: u16) -> Result<()> {
    eprintln!("Please select a network interface manually:");
    let interfaces = pnet::datalink::interfaces();
    let options = interfaces.iter().map(|x| {
        (
            &x.name,
            x.ips
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    });
    for x in options.enumerate() {
        println!("{}. {:?}", x.0, x.1);
    }

    eprint!("Input: ");
    stderr().flush()?;

    let input = stdin().lines().next().unwrap().unwrap();
    let input = input.parse::<u8>().map_err(|_| Error::InvalidSelect)?;

    let interface: &NetworkInterface = interfaces
        .get(input as usize)
        .map_or_else(|| Err(Error::InvalidSelect), Ok)?;

    let text = format!("{}:{}", interface.ips[0].ip(), port);
    qr2term::print_qr(&text).unwrap();
    Ok(())
}
