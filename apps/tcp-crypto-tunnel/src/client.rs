use crate::errors::*;
use crate::generic_main;
use clap::ArgMatches;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let server_addr = matches.get_one::<String>("server").unwrap().clone();
    let port = *matches.get_one::<u16>("port").unwrap();

    generic_main(server_addr, port)
}
