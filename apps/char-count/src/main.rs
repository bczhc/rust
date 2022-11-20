use std::io::stdin;

use bczhc_lib::char::CharReader;
use clap::{Arg, ArgAction, Command};
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let matches = Command::new("char-count")
        .arg(
            Arg::new("grapheme")
                .short('g')
                .long("grapheme")
                .help("Count Unicode grapheme clusters")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let chars = CharReader::new(stdin().lock());
    let grapheme = matches.get_flag("grapheme");

    let length = if grapheme {
        String::from_iter(chars).graphemes(true).count()
    } else {
        chars.count()
    };
    println!("{}", length);
}
