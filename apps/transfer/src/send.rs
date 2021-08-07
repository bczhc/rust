use clap::ArgMatches;
use crate::lib::handle_config;
use std::fs::{File, Permissions};
use std::io::{Write, Read, BufWriter};
use lib::io::OpenOrCreate;

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    // send:
    // transfer send [-v] [-f <file>...]

    let verbose = matches.is_present("verbose");
    let file = matches.value_of("file");

    let configuration = handle_config();

    println!("{:?}", configuration);

    Ok(())
}
