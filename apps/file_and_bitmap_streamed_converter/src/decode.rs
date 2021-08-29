use bczhc_lib::point::PointU32;
use bmp::Pixel;
use byteorder::{LittleEndian, ReadBytesExt};
use clap::{App, Arg};
use file_bitmap_codec::errors::Result;
use std::fs::OpenOptions;
use std::io::{BufWriter, Cursor, Write};

fn main() -> Result<()> {
    let matches = App::new("bmp2file")
        .about("This can convert a bitmap into a file")
        .author("bczhc <bczhc0@126.com>")
        .arg(
            Arg::with_name("src")
                .value_name("src-path")
                .help("The bitmap path")
                .required(true),
        )
        .arg(
            Arg::with_name("dest")
                .value_name("dest-path")
                .help("The output file path")
                .required(true),
        )
        .get_matches();

    let input_path = matches.value_of("src").unwrap();
    let output_path = matches.value_of("dest").unwrap();

    file_bitmap_codec::decode(input_path, output_path)?;

    println!("Done");

    Ok(())
}
