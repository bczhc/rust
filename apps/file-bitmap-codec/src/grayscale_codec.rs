use crate::Result;
use bczhc_lib::io::OpenOrCreate;
use byteorder::{ByteOrder, LittleEndian};
use image::io::Reader;
use image::GrayImage;
use image::ImageFormat;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub fn encode(src: &str, dest: &str) -> Result<()> {
    let file = File::open(src)?;
    let file_size = file.metadata()?.len();
    let size = file_size + 8 /* packed 8-byte file-size integer */ ;

    let width = f64::sqrt(size as f64) as u64 + 1;
    let height = width;
    assert!(width * height >= size);

    let mut packed_file_size = [0_u8; 8];
    LittleEndian::write_u64(&mut packed_file_size, file_size);

    let mut image = GrayImage::new(width as u32, height as u32);
    let mut pixels_mut = image.pixels_mut();

    for b in packed_file_size {
        pixels_mut.next().unwrap().0[0] = b;
    }

    let reader = BufReader::new(file);
    for b in reader.bytes() {
        pixels_mut.next().unwrap().0[0] = b?;
    }

    image.save_with_format(dest, ImageFormat::Bmp)?;

    println!("Done");
    Ok(())
}

pub fn decode(src: &str, dest: &str) -> Result<()> {
    let mut reader = Reader::new(BufReader::new(File::open(src)?));
    reader.no_limits();
    reader.set_format(ImageFormat::Bmp);
    let image = reader.decode()?.into_luma8();
    let mut pixels = image.pixels();

    let mut packed_file_size = [0_u8; 8];
    for b in &mut packed_file_size {
        *b = pixels.next().unwrap().0[0];
    }

    let file_size = LittleEndian::read_u64(&packed_file_size);

    let mut writer = BufWriter::new(File::open_or_create(dest)?);
    for _ in 0..file_size {
        writer.write_all(&pixels.next().unwrap().0)?;
    }
    println!("Done");
    Ok(())
}
