use clap::{Arg, Command};

use image_track::AnyResult;

fn main() -> AnyResult<()> {
    let matches = Command::new("image-track")
        .subcommand(
            Command::new("new")
                .arg(Arg::new("img").required(true))
                .arg(Arg::new("force").short('f').long("force"))
                .about("Initialize a new empty image"),
        )
        .subcommand(
            Command::new("create")
                .arg(Arg::new("path").multiple_values(true).required(true))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("The output file"),
                )
                .about("Directly write an image from files, default to stdout"),
        )
        .subcommand(
            Command::new("info")
                .arg(Arg::new("img").required(true))
                .arg(
                    Arg::new("json")
                        .short('j')
                        .long("json")
                        .help("Print as JSON"),
                )
                .about("Print the information of an image"),
        )
        .subcommand(
            Command::new("add")
                .arg(Arg::new("img").required(true))
                .arg(Arg::new("path").multiple_values(true).required(true))
                .about("Add a track from a file to an image"),
        )
        .subcommand_required(true)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        image_track::new::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("info") {
        image_track::info::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("add") {
        image_track::add::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("create") {
        image_track::create::main(matches)
    } else {
        unreachable!()
    }
}
