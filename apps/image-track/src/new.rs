use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};
use clap::ArgMatches;

use crate::{AnyResult, ImageInfo, HEADER, SECTOR_SIZE};

pub fn main(matches: &ArgMatches) -> AnyResult<()> {
    let img = matches.value_of("img").unwrap();
    let force = matches.is_present("force");

    if Path::new(img).exists() && !force {
        return Err("File exists. Use `--force` to overwrite".into());
    }
    let mut file = File::options()
        .truncate(true)
        .create(true)
        .write(true)
        .open(img)?;

    let image_info = ImageInfo::new(SECTOR_SIZE, Vec::new());

    let image_info_json = serde_json::to_string(&image_info).unwrap();

    file.write_all(HEADER)?;
    file.write_u32::<LittleEndian>(image_info_json.len() as u32)?;

    file.write_all(image_info_json.as_bytes())?;

    println!("Done");

    Ok(())
}
