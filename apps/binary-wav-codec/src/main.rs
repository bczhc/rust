#![allow(const_evaluatable_unchecked)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bczhc_lib::io::OpenOrCreate;
use binary_wav_codec::errors::Error;
use binary_wav_codec::*;
use clap::{Arg, ArgMatches, Command};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

fn main() -> Result<()> {
    let matches = Command::new("binary-wav-codec")
        .author("bczhc <bczhc0@126.com>")
        .about("Codec between binary data and .wav files")
        .subcommand(
            Command::new("encode")
                .arg(Arg::new("src").required(true).takes_value(true).id("src"))
                .arg(Arg::new("dest").required(true).takes_value(true).id("dest")),
        )
        .subcommand(
            Command::new("decode")
                .arg(Arg::new("src").required(true).takes_value(true).id("src"))
                .arg(Arg::new("dest").required(true).takes_value(true).id("dest")),
        )
        .get_matches();

    let subcommand = matches.subcommand();
    if subcommand.is_none() {
        return Err(Error::UsageError(String::from("Subcommand needed")));
    }

    match subcommand.unwrap() {
        ("encode", matches) => handle_encode(matches),
        ("decode", matches) => handle_decode(matches),
        _ => {
            unreachable!();
        }
    }
}

fn open_buffered_reader<P>(f: P) -> Result<BufReader<File>>
where
    P: AsRef<Path>,
{
    Ok(BufReader::new(File::open(f)?))
}

fn open_buffered_writer<P>(f: P) -> Result<BufWriter<File>>
where
    P: AsRef<Path>,
{
    Ok(BufWriter::new(File::open_or_create(f)?))
}

fn handle_encode(matches: &ArgMatches) -> Result<()> {
    let src = matches.value_of("src").unwrap();
    let dest = matches.value_of("dest").unwrap();

    let mut reader = open_buffered_reader(src)?;
    let mut writer = open_buffered_writer(dest)?;

    encode_stream(&mut reader, &mut writer)?;

    Ok(())
}
fn handle_decode(matches: &ArgMatches) -> Result<()> {
    let src = matches.value_of("src").unwrap();
    let dest = matches.value_of("dest").unwrap();

    let mut reader = open_buffered_reader(src)?;
    let mut writer = open_buffered_writer(dest)?;

    decode_stream(&mut reader, &mut writer)?;

    Ok(())
}
