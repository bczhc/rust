use bmp::{open, Image, Pixel};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::LinkedList;
use std::fs::File;
use std::io::{Cursor, Read, Seek, BufReader};

fn main() -> Result<(), ()> {
    let path = "/home/bczhc/a.bmp";
    let file = File::open(path).unwrap();
    let len = file.metadata().unwrap().len() as u32 + 8;
    let mut input = BufReader::new(file);

    let mut len_byte = vec![0; 6];
    (&mut len_byte[0..4]).write_u32::<LittleEndian>(len);

    let pixel_count = multiple_of_3(len);
    let width = f64::sqrt(pixel_count as f64) as u32;
    let height = pixel_count / width + 1 + 2;
    assert!(width * height >= pixel_count + 2);

    let mut image = Image::new(width, height);

    image.set_pixel(0, 0, three_bytes_to_a_pixel(&len_byte[0..3]));
    image.set_pixel(1, 0, three_bytes_to_a_pixel(&len_byte[3..]));

    'a: for y in 0..height {
        for x in (if y == 0 { 2 } else { 0 })..width {
            let pixel = read_a_byte_to_pixel(&mut input);
            if let Err(_e) = pixel {
                break 'a;
            }
            let pixel = pixel.unwrap();
            image.set_pixel(x, y, pixel);
        }
    }

    println!("Done");
    image.save("/home/bczhc/b.bmp");

    Ok(())
}

#[inline]
fn three_bytes_to_a_pixel(bytes: &[u8]) -> Pixel {
    Pixel::new(bytes[0], bytes[1], bytes[2])
}

#[inline]
fn multiple_of_3(len: u32) -> u32 {
    if len % 3 == 0 {
        len / 3
    } else {
        len / 3 + 1
    }
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
