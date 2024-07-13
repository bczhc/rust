use std::fs::File;
use std::io::{BufReader, BufWriter, stdin};
use clap::Parser;
use wav_wrapper::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let reader = BufReader::new(stdin().lock());
    let file = File::options()
        .write(true)
        .read(true)
        .truncate(true)
        .create(true)
        .open(&args.output_file)?;
    let writer = BufWriter::new(file);

    // WavWriter::new(writer, WavSpec {
    //     channels: args.channels,
    //     sample_rate: args.sample_rate,
    //     bits_per_sample: 16,
    //     sample_format: SampleFormat::,
    // });
    Ok(())
}
