use bmp::{Image, Pixel};
use byteorder::{LittleEndian, WriteBytesExt};

use clap::{App, Arg};
use bczhc_lib::utils::get_args_without_self_path;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() -> Result<(), ()> {
    let matches = App::new("file2bmp")
        .about("This can convert a file into a bitmap")
        .author("bczhc <bczhc0@126.com>")
        .arg(
            Arg::with_name("src")
                .value_name("src-path")
                .help("The file path")
                .required(true),
        )
        .arg(
            Arg::with_name("dest")
                .value_name("dest-path")
                .help("The output bitmap path")
                .required(true),
        )
        .get_matches();

    let input_path = matches.value_of("src").unwrap();
    let output_path = matches.value_of("dest").unwrap();

    let file = File::open(input_path).unwrap();
    let len = file.metadata().unwrap().len() as u32 + 8;
    let mut input = BufReader::new(file);

    let mut len_byte = vec![0; 6];
    (&mut len_byte[0..4])
        .write_u32::<LittleEndian>(len)
        .unwrap();

    let pixel_count = if len % 3 == 0 { len / 3 } else { len / 3 + 1 };
    let width = f64::sqrt(pixel_count as f64) as u32;
    let height = pixel_count / width + 1 + 1;
    assert!(width * height >= pixel_count + width);

    let mut image = Image::new(width, height);

    image.set_pixel(0, 0, three_bytes_to_a_pixel(&len_byte[0..3]));
    image.set_pixel(1, 0, three_bytes_to_a_pixel(&len_byte[3..]));

    let mut iter = image.coordinates();
    iter.nth((width - 1) as usize);

    for (x, y) in iter {
        let pixel = read_a_byte_to_pixel(&mut input);
        if let Err(_e) = pixel {
            break;
        }
        let pixel = pixel.unwrap();
        image.set_pixel(x, y, pixel);
    }

    println!("Done");
    image.save(output_path).unwrap();

    Ok(())
}

#[inline]
fn three_bytes_to_a_pixel(bytes: &[u8]) -> Pixel {
    Pixel::new(bytes[0], bytes[1], bytes[2])
}

#[inline]
fn read_a_byte_to_pixel<T>(file: &mut T) -> std::io::Result<Pixel>
where
    T: Read,
{
    let mut buf = [0_u8; 3];
    file.read_exact(&mut buf)?;
    Ok(three_bytes_to_a_pixel(&buf))
}
