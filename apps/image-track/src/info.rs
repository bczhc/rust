use clap::ArgMatches;

use crate::{open_image, AnyResult};

pub fn main(matches: &ArgMatches) -> AnyResult<()> {
    let img = matches.value_of("img").unwrap();

    let (file, image_info) = open_image(img)?;
    drop(file);

    if matches.is_present("json") {
        println!("{}", serde_json::to_string_pretty(&image_info).unwrap());
    } else {
        println!("{:#?}", image_info);
    }
    Ok(())
}
