use crate::Result;
use bczhc_lib::io::OpenOrCreate;
use image::GrayImage;
use image::ImageFormat;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub fn encode(src: &str, dest: &str) -> Result<()> {
    let file = File::open(src)?;
    let size = file.metadata()?.len();
    let width = f64::sqrt(size as f64) as u64 + 1;
    let height = width;
    assert!(width * height >= size);

    let mut image = GrayImage::new(width as u32, height as u32);
    let mut pixels_mut = image.pixels_mut();

    let reader = BufReader::new(file);
    for b in reader.bytes() {
        pixels_mut.next().unwrap().0[0] = b?;
    }

    image.save_with_format(dest, ImageFormat::Bmp)?;

    println!("Done");
    Ok(())
}

pub fn decode(src: &str, dest: &str) -> Result<()> {
    let mut reader = BufReader::new(File::open(src)?);

    let mut writer = BufWriter::new(File::open_or_create(dest)?);

    let image = image::load(&mut reader, ImageFormat::Bmp)?.into_luma8();
    for x in image.pixels() {
        writer.write_all(&x.0)?;
    }

    println!("Done");
    Ok(())
}
