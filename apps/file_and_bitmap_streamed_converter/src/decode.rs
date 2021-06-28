use bmp::Pixel;
use byteorder::{LittleEndian, ReadBytesExt};
use lib::point::PointU32;
use std::fs::OpenOptions;
use std::io::{BufWriter, Cursor, Write};
use lib::utils::get_args_without_self_path;

fn main() -> Result<(), ()> {
    let args = get_args_without_self_path();
    let output_path = &args[0];
    let input_path = &args[1];

    let mut output = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(output_path)
            .unwrap(),
    );

    let image = bmp::open(input_path).unwrap();

    let width = image.get_width();
    let height = image.get_height();
    assert!(width >= 2 && height >= 1);

    let p1 = image.get_pixel(0, 0);
    let p2 = image.get_pixel(1, 0);
    let _file_size = resolve_file_size(p1, p2);

    let mut positioner = Positioner::new(width, height);
    positioner.nth((width - 1) as usize);

    for point in positioner {
        let pixel = image.get_pixel(point.x, point.y);
        output.write(&[pixel.r, pixel.g, pixel.b]).unwrap();
    }

    output.flush().unwrap();
    println!("Done");

    Ok(())
}

fn resolve_file_size(p1: Pixel, p2: Pixel) -> u32 {
    let vec = vec![p1.r, p1.g, p1.b, p2.r, 0, 0];
    let mut cursor = Cursor::new(vec);
    cursor.read_u32::<LittleEndian>().unwrap()
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
