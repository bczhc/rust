use std::fs::{remove_file, File};
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;

use pbr::ProgressBar;

use ucd_parser::cli::build_cli;
use ucd_parser::parse_xml;

static UCD_URL: &str = "https://www.unicode.org/Public/UCD/latest/ucdxml/ucd.all.flat.zip";

async fn download<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let response = reqwest::get(UCD_URL).await.unwrap();
    let size = response.content_length().unwrap();

    let mut progress_bar = ProgressBar::new(size);

    let file = File::options()
        .read(true)
        .write(true)
        .append(false)
        .truncate(true)
        .open(path)?;
    let mut writer = BufWriter::new(file);

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    while let Some(b) = stream.next().await {
        let chunk = b.unwrap();
        writer.write_all(chunk.as_ref()).unwrap();
        progress_bar.add(chunk.len() as u64);
    }
    progress_bar.finish_println("Done\n");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let output_path = matches.get_one::<String>("output").unwrap();

    let temp_file = mktemp::Temp::new_file()?;

    let output_path = Path::new(output_path);
    if output_path.exists() {
        remove_file(output_path)?;
    }

    download(&temp_file).await?;

    parse_xml(&temp_file, output_path, |x, p| {
        println!("{:?}", (x, p));
    })?;

    Ok(())
}
