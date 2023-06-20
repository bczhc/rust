use anyhow::anyhow;
use clap::Parser;
use float_cmp::approx_eq;
use image::{GenericImageView, GrayAlphaImage, ImageBuffer, ImageFormat, LumaA, Pixel};

use image_stegano_bg::cli::CliArgs;

// alpha = 1 - img1 + img2
// result = img2 / alpha
fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    println!("Loading...");
    let image1 = image::open(&args.image1)?;
    let image2 = image::open(&args.image2)?;

    if image1.dimensions() != image2.dimensions() {
        return Err(anyhow!("Image dimensions should be identical"));
    }
    let dimensions = image1.dimensions();

    let image1 = image1.grayscale();
    let image2 = image2.grayscale();

    image1.save("./l1.png")?;
    image2.save("./l2.png")?;

    let mut result: GrayAlphaImage = ImageBuffer::new(dimensions.0, dimensions.1);

    println!("Processing...");
    for (x, y, p) in image1.pixels() {
        let luma1 = p.to_luma().0[0] as f64 / u8::MAX as f64;
        let mut luma2 = image2.get_pixel(x, y).to_luma().0[0] as f64 / u8::MAX as f64;

        // // image1 should be brighter than image2
        // if luma2 > luma1 {
        //     luma2 = luma1;
        // }

        let out_a = 1.0 - luma1 + luma2;
        let out_l = if approx_eq!(f64, out_a, 0.0, epsilon = 0.0001) {
            // arbitrarily pick a value because the opacity is zero
            0.0
        } else {
            luma2 / out_a
        };

        result.put_pixel(
            x,
            y,
            LumaA([
                (out_l * u8::MAX as f64) as u8,
                (out_a * u8::MAX as f64) as u8,
            ]),
        )
    }

    println!("Saving...");
    result.save_with_format(&args.output, ImageFormat::Png)?;

    println!("Done");
    Ok(())
}
