use crate::{open_image, AnyResult};
use clap::ArgMatches;

pub fn main(matches: &ArgMatches) -> AnyResult<()> {
    let img = matches.value_of("img").unwrap();
    let paths = matches.values_of("path").unwrap();

    let (file, image_info) = open_image(img)?;

    Ok(())
}
