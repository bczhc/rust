use crate::{open_image, AnyResult};
use clap::ArgMatches;

pub fn main(matches: &ArgMatches) -> AnyResult<()> {
    let img = matches.value_of("img").unwrap();
    let _paths = matches.values_of("path").unwrap();

    let (_file, _image_info) = open_image(img)?;

    Ok(())
}
