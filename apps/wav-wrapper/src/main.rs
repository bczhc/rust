#![feature(yeet_expr)]

use anyhow::anyhow;
use std::fs::File;
use std::io::{stdin, BufReader, BufWriter, Read, Seek, Write};
use std::sync::Mutex;

use clap::Parser;
use hound::{SampleFormat, WavSpec, WavWriter};
use once_cell::sync::Lazy;
use yeet_ops::yeet;

use stream_num_reader::{read_types, NumberReader};
use wav_wrapper::Args;

static ARGS: Lazy<Mutex<Args>> = Lazy::new(|| Mutex::new(Default::default()));

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    *ARGS.lock().unwrap() = args.clone();

    let reader = BufReader::new(stdin().lock());
    let file = File::options()
        .write(true)
        .read(true)
        .truncate(true)
        .create(true)
        .open(&args.output_file)?;
    let writer = BufWriter::new(file);

    let mut wav_writer = create_wav_writer(writer, &args.output_format)?;

    macro_rules! write_samples {
        ($rt:ty) => {{
            let nums = NumberReader::<_, $rt>::new(reader);
            for x in nums {
                let sample = x?;
                wav_writer.write_sample(sample)?;
            }
        }};
    }

    match args.input_format.as_str() {
        "s8" => write_samples!(read_types::S8),
        "s16le" => write_samples!(read_types::S16LE),
        "s16be" => write_samples!(read_types::S16LE),
        "s32le" => write_samples!(read_types::S32LE),
        "s32be" => write_samples!(read_types::S32BE),
        "f32le" => write_samples!(read_types::F32LE),
        "f32be" => write_samples!(read_types::F32BE),
        _ => {
            yeet!(anyhow!("Unknown input format: {}", args.input_format));
        }
    }

    wav_writer.finalize()?;

    Ok(())
}

fn create_wav_writer<W: Write + Seek>(writer: W, format: &str) -> anyhow::Result<WavWriter<W>> {
    let args = ARGS.lock().unwrap();

    let (bit_depth, sample_format) = match format {
        "s8" => (8, SampleFormat::Int),
        "s16" => (16, SampleFormat::Int),
        "s32" => (32, SampleFormat::Int),
        "f32" => (32, SampleFormat::Float),
        _ => {
            yeet!(anyhow!("Unknown output format: {}", format))
        }
    };

    let wav_writer = WavWriter::new(
        writer,
        WavSpec {
            channels: args.channels,
            sample_rate: args.sample_rate,
            bits_per_sample: bit_depth,
            sample_format,
        },
    )?;
    Ok(wav_writer)
}
