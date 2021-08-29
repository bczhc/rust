use bmp::{Image, Pixel};
use byteorder::{LittleEndian, WriteBytesExt};

use bczhc_lib::utils::get_args_without_self_path;
use clap::{App, Arg};
use file_bitmap_codec::errors::Result;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() -> Result<()> {
    let matches = App::new("file2bmp")
        .about("This can convert a file into a bitmap")
        .author("bczhc <bczhc0@126.com>")
        .arg(
            Arg::with_name("src")
                .value_name("src-path")
                .help("The file path")
                .required(true),
        )
        .arg(
            Arg::with_name("dest")
                .value_name("dest-path")
                .help("The output bitmap path")
                .required(true),
        )
        .get_matches();

    let input_path = matches.value_of("src").unwrap();
    let output_path = matches.value_of("dest").unwrap();

    file_bitmap_codec::encode(input_path, output_path)?;

    println!("Done");

    Ok(())
}
