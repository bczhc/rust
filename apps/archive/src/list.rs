use clap::ArgMatches;
use std::fs::File;
use std::io::Read;
use std::mem::size_of_val;

use crate::errors::{Error, Result};
use crate::{Header, ReadFrom, FILE_MAGIC};

pub fn main(matches: &ArgMatches) -> Result<()> {
    let archive = matches.value_of("archive").unwrap();

    let mut file = File::open(archive)?;

    let header = Header::read_from(&mut file)?;
    if &header.magic_number != FILE_MAGIC {
        return Err(Error::InvalidFileType);
    }

    println!(
        "\
Version: {}
Content offset: {}",
        header.version, header.content_offset
    );

    Ok(())
}
