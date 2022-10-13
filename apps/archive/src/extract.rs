use crate::errors::*;
use clap::ArgMatches;
use std::fs::File;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive_path = matches.get_one::<String>("archive").unwrap();
    let file = File::open(archive_path)?;

    Ok(())
}
