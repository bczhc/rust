use crate::errors::*;
use crate::reader::ArchiveReader;
use crate::{Header, Info};
use clap::ArgMatches;

pub fn main(matches: &ArgMatches) -> Result<()> {
    let path = matches.get_one::<String>("archive").unwrap();
    let archive = ArchiveReader::new(path)?;

    print_info(&archive.header)?;

    Ok(())
}

pub fn print_info(header: &Header) -> Result<()> {
    let info: Info = serde_json::from_str(&header.info_json).map_err(|_| Error::InvalidInfoJson)?;
    println!("{}\n{}", header, info);
    Ok(())
}
