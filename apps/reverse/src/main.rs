extern crate bczhc_lib;

use bczhc_lib::io::ReadLines;
use clap::{Arg, ArgAction, Command};
use std::io::{stdin, Read};
use unicode_segmentation::UnicodeSegmentation;

enum ReverseMode {
    Line,
    All,
}

fn main() -> Result<(), String> {
    let matches = Command::new("reverse")
        .arg(
            Arg::new("line")
                .short('l')
                .long("line")
                .conflicts_with("all")
                .action(ArgAction::SetTrue)
                .help("Reverse string by each line"),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .conflicts_with("line")
                .action(ArgAction::SetTrue)
                .help("Reverse all string read from stdin; this is the default mode"),
        )
        .arg(
            Arg::new("grapheme")
                .short('g')
                .long("grapheme")
                .help("Reverse by Unicode grapheme clusters")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let mut reverse_mode = ReverseMode::Line;
    if matches.get_flag("all") {
        reverse_mode = ReverseMode::All
    }

    let grapheme = matches.get_flag("grapheme");

    let mut stdin = stdin().lock();

    match reverse_mode {
        ReverseMode::All => {
            let mut read = String::new();
            stdin.read_to_string(&mut read).unwrap();
            println!("{}", reverse_string(&read, grapheme));
        }
        ReverseMode::Line => {
            for line in stdin.lines() {
                println!("{}", reverse_string(&line, grapheme));
            }
        }
    }

    Ok(())
}

fn reverse_string(s: &str, grapheme_cluster: bool) -> String {
    if grapheme_cluster {
        s.graphemes(true).rev().collect()
    } else {
        s.chars().rev().collect()
    }
}
