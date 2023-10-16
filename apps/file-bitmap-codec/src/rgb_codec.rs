use crate::Result;
use bmp::{Image, Pixel};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use euclid::default::Point2D;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Cursor, Read, Write};

pub fn encode(input_path: &str, output_path: &str) -> Result<()> {
    let file = File::open(input_path)?;
    let len = file.metadata()?.len() as u32 + 8;
    let mut input = BufReader::new(file);

    let mut len_byte = [0; 6];
    (&mut len_byte[0..4]).write_u32::<LittleEndian>(len)?;

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
        let pixel = pixel?;
        image.set_pixel(x, y, pixel);
    }

    image.save(output_path)?;
    println!("Done");
    Ok(())
}

pub fn decode(input_path: &str, output_path: &str) -> Result<()> {
    let mut output = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(output_path)?,
    );

    let image = bmp::open(input_path)?;

    let width = image.get_width();
    let height = image.get_height();
    assert!(width >= 2 && height >= 1);

    let p1 = image.get_pixel(0, 0);
    let p2 = image.get_pixel(1, 0);
    let _file_size = resolve_file_size(p1, p2)?;

    let mut positioner = Positioner::new(width, height);
    positioner.nth((width - 1) as usize);

    for point in positioner {
        let pixel = image.get_pixel(point.x, point.y);
        output.write_all(&[pixel.r, pixel.g, pixel.b])?;
    }

    output.flush()?;
    println!("Done");
    Ok(())
}

fn resolve_file_size(p1: Pixel, p2: Pixel) -> std::io::Result<u32> {
    let vec = vec![p1.r, p1.g, p1.b, p2.r, 0, 0];
    let mut cursor = Cursor::new(vec);
    cursor.read_u32::<LittleEndian>()
}

struct Positioner {
    width: u32,
    height: u32,
    width_i: u32,
    height_i: u32,
}

impl Positioner {
    #[inline]
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            width_i: 0,
            height_i: 0,
        }
    }
}

type PointU32 = Point2D<u32>;

impl Iterator for Positioner {
    type Item = PointU32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.height_i == self.height {
            return None;
        }

        let w = self.width_i;
        let h = self.height_i;
        self.width_i += 1;
        if self.width_i == self.width {
            self.width_i = 0;
            self.height_i += 1;
        }
        Some(PointU32::new(w, h))
    }
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
