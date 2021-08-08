//! ## Structure:
//!
//!
//! ### Header (8):
//! | "bczhc" (5) | EndMark (1) | Type (1) | Null (1) |
//!
//! ### End:
//! | Header (8) |
//!
//! ### Not end:
//! #### File:
//! | Header (8) | FilenameLength (4) | Filename | ContentLength (4) | Digest | Content |
//! #### Directory
//! | Header (8) | PathLength (4) | Path |
//! #### Stdin
//! | Header (8) | ContentLength (4) | Digest | Content |
//!
//!
use clap::{App, Arg};
use std::fs::{create_dir, DirEntry, File};
use transfer::{Error, MyResult};

fn main() -> MyResult<()> {
    // transfer <subcommand>
    // subcommands: send, receive

    let matches = App::new("transfer")
        .about("A tool to send and receive files or texts")
        .author("bczhc <bczhc0@126.com>")
        .subcommand(
            App::new("send")
                .about("Send files or texts")
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .help("Show verbose messages")
                        .required(false),
                )
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .help("Specify the files to be transferred")
                        .takes_value(true)
                        .required(false)
                        .multiple(true),
                ),
        )
        .subcommand(
            App::new("receive").about("Receive files or texts").arg(
                Arg::with_name("verbose")
                    .required(false)
                    .short("v")
                    .long("verbose"),
            ),
        )
        .get_matches();

    let subcommand = matches.subcommand();
    match subcommand.0 {
        "send" => transfer::send::run(subcommand.1.unwrap()),
        "receive" => transfer::receive::run(subcommand.1.unwrap()),
        _ => {
            println!("{}", matches.usage());
            Err(Error::UnknownSubcommand)
        }
    }
}
