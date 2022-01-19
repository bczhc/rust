#![feature(generic_const_exprs)]

use bczhc_lib::io::OpenOrCreate;
use binary_wav_codec::errors::Error;
use binary_wav_codec::*;
use clap::{App, Arg, ArgMatches};
use hound::WavWriter;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

fn main() -> Result<()> {
    let matches = App::new("binary-wav-codec")
        .author("bczhc <bczhc0@126.com>")
        .about("Codec between binary data and .wav files")
        .subcommand(
            App::new("encode")
                .arg(Arg::new("src").required(true).takes_value(true).name("src"))
                .arg(
                    Arg::new("dest")
                        .required(true)
                        .takes_value(true)
                        .name("dest"),
                ),
        )
        .subcommand(
            App::new("decode")
                .arg(Arg::new("src").required(true).takes_value(true).name("src"))
                .arg(
                    Arg::new("dest")
                        .required(true)
                        .takes_value(true)
                        .name("dest"),
                ),
        )
        .get_matches();

    let subcommand = matches.subcommand();
    if let None = subcommand {
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
