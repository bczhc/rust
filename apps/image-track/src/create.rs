use crate::{
    pad_sector, AnyResult, ImageInfo, PadZero, Track, HEADER, HEAD_INFO_SIZE, SECTOR_SIZE,
};
use bczhc_lib::io::OpenOrCreate;
use byteorder::{LittleEndian, WriteBytesExt};
use clap::ArgMatches;
use std::fs::File;
use std::io;
use std::io::{stdout, BufWriter, Write};
use std::mem::size_of;
use std::path::Path;

pub fn main(matches: &ArgMatches) -> AnyResult<()> {
    let paths = matches.values_of("path").unwrap();
    let output = matches.value_of("output");

    match output {
        None => {
            write_image(&mut stdout().lock(), paths)?;
        }
        Some(output) => {
            let file = File::open_or_create(output)?;
            let mut writer = BufWriter::new(file);
            write_image(&mut writer, paths)?;
        }
    }

    Ok(())
}

fn write_image<'a, W, I>(writer: &mut W, paths: I) -> AnyResult<()>
where
    W: Write,
    I: Iterator<Item = &'a str>,
{
    let paths = paths.collect::<Vec<_>>();

    writer.write_all(HEADER)?;

    assert_eq!(HEAD_INFO_SIZE % SECTOR_SIZE as u64, 0);

    let content_start_sector = HEAD_INFO_SIZE / SECTOR_SIZE as u64;

    let mut tracks = Vec::new();

    let mut start_sector = content_start_sector;
    for (i, path) in paths.iter().enumerate() {
        let path = Path::new(path);
        let size = path.metadata()?.len();
        let used_sectors = pad_sector(size);
        let track = Track {
            number: (i + 1) as u32,
            start_sector,
            end_sector: start_sector + used_sectors - 1,
            used_sectors,
            content_size: size,
        };
        start_sector += used_sectors;
        tracks.push(track);
    }

    let image_info = ImageInfo::new(SECTOR_SIZE, tracks);
    let json = serde_json::to_string(&image_info).unwrap();

    // restrict the json info and some headers can only use 0..u16::MAX area
    // otherwise it may overlap the content area
    let head_info_len = json.len() + HEADER.len() + size_of::<u32>();
    if head_info_len >= u16::MAX as usize {
        return Err("Too many tracks! (info size > 32 sectors)".into());
    }

    writer.write_u32::<LittleEndian>(json.len() as u32)?;
    writer.write_all(json.as_bytes())?;

    // pad the info area
    let pad_size = HEAD_INFO_SIZE - head_info_len as u64;
    writer.pad_zero(pad_size)?;

    for (track, path) in image_info.tracks.iter().zip(&paths) {
        let mut file = File::open(path)?;
        let size = io::copy(&mut file, writer)?;
        assert_eq!(size, track.content_size);

        let pad_size = track.used_sectors * SECTOR_SIZE as u64 - track.content_size;
        writer.pad_zero(pad_size)?;
    }

    Ok(())
}
