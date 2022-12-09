use crate::errors::*;
use crate::generic_main;

use clap::ArgMatches;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let address = matches.get_one::<String>("address").unwrap().clone();
    let port = *matches.get_one::<u16>("port").unwrap();

    generic_main(address, port)
}
