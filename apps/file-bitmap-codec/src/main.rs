use clap::{Arg, Command, PossibleValue};
use file_bmp_codec::Result;
use file_bmp_codec::*;

fn main() -> Result<()> {
    let grayscale_possible_values = [
        (PossibleValue::new("rgb"), CodecType::Rgb),
        (
            PossibleValue::new("grayscale").aliases(["gray", "gs"]),
            CodecType::Grayscale,
        ),
    ];

    let matches = Command::new("file-bmp-codecs")
        .subcommand_required(true)
        .author("bczhc <bczhc0@126.com>")
        .about("Codecs between binary files and bitmaps")
        .subcommand(
            Command::new("encode")
                .arg(
                    Arg::new("src")
                        .value_name("src")
                        .help("File path")
                        .required(true),
                )
                .arg(
                    Arg::new("dest")
                        .value_name("dest")
                        .help("Output bitmap path")
                        .required(true),
                )
                .arg(
                    Arg::new("type")
                        .takes_value(true)
                        .long("type")
                        .short('t')
                        .possible_values(grayscale_possible_values.clone().map(|x| x.0))
                        .ignore_case(true)
                        .default_value("rgb"),
                ),
        )
        .subcommand(
            Command::new("decode")
                .arg(
                    Arg::new("src")
                        .value_name("src")
                        .help("Bitmap path")
                        .required(true),
                )
                .arg(
                    Arg::new("dest")
                        .value_name("dest")
                        .help("Output file path")
                        .required(true),
                )
                .arg(
                    Arg::new("type")
                        .takes_value(true)
                        .long("type")
                        .short('t')
                        .possible_values(grayscale_possible_values.clone().map(|x| x.0))
                        .ignore_case(true)
                        .default_value("rgb"),
                ),
        )
        .get_matches();

    let subcommand = matches.subcommand().unwrap();
    let src = subcommand.1.value_of("src").unwrap();
    let dest = subcommand.1.value_of("dest").unwrap();

    let codec_type = grayscale_possible_values
        .clone()
        .iter()
        .find(|x| x.0.matches(subcommand.1.value_of("type").unwrap(), true))
        .unwrap()
        .1
        .clone();

    match (codec_type, subcommand.0) {
        (CodecType::Rgb, "encode") => rgb_codec::encode(src, dest),
        (CodecType::Rgb, "decode") => rgb_codec::decode(src, dest),
        (CodecType::Grayscale, "encode") => grayscale_codec::encode(src, dest),
        (CodecType::Grayscale, "decode") => grayscale_codec::decode(src, dest),
        _ => {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
enum CodecType {
    Rgb,
    Grayscale,
}
