use std::io;
use std::io::{Read, Write};
use std::mem::transmute_copy;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::thread::spawn;

use bytesize::ByteSize;

use crate::errors::Result;
use crate::{Compression, Error};

#[derive(Copy, Clone)]
pub enum Level {
    Best,
    Numeric(u32),
}

impl Level {
    pub fn to_numeric(&self, compression: Compression) -> u32 {
        match self {
            Level::Best => compression.best_level(),
            Level::Numeric(n) => *n,
        }
    }
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(level) => Ok(Self::Numeric(level)),
            Err(_) => {
                if s == "best" {
                    Ok(Self::Best)
                } else {
                    Err(())
                }
            }
        }
    }
}

pub fn create_compressor(method: Compression, level: Level) -> Box<dyn Compress> {
    let level = level.to_numeric(method);

    match method {
        Compression::Gzip => Box::new(GzipCompressor::new(level)),
        Compression::Xz => Box::new(XzCompressor::new(level)),
        Compression::Zstd => Box::new(ZstdCompressor::new(level)),
        Compression::Bzip2 => Box::new(Bzip2Compressor::new(level)),
        Compression::None => Box::new(NoCompressor::new()),
        Compression::Brotli => Box::new(BrotliCompressor::new(level)),
        Compression::Bzip3 => Box::new(Bzip3Compressor::new(Bzip3Compressor::level_to_block_size(
            level,
        ))),
        Compression::External => {
            unreachable!("Invalid argument")
        }
    }
}

pub fn create_decompressor(method: Compression) -> Box<dyn Decompress> {
    match method {
        Compression::Gzip => Box::new(GzipDecompressor),
        Compression::Xz => Box::new(XzDecompressor),
        Compression::Zstd => Box::new(ZstdDecompressor),
        Compression::Bzip2 => Box::new(Bzip2Decompressor),
        Compression::None => Box::new(NoDecompressor),
        Compression::Brotli => Box::new(BrotliDecompressor),
        Compression::Bzip3 => Box::new(Bzip3Decompressor),
        Compression::External => {
            unreachable!("Invalid argument")
        }
    }
}

pub trait Compress {
    /// Returns the size after compression
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64>;
}

pub trait Decompress {
    /// Returns size of uncompressed content
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64>;
}

pub struct GzipCompressor {
    level: flate2::Compression,
}

impl Compress for GzipCompressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = flate2::read::GzEncoder::new(from, self.level);
        Ok(io::copy(&mut encoder, to)?)
    }
}

impl GzipCompressor {
    pub fn new(level: u32) -> Self {
        Self {
            level: flate2::Compression::new(level),
        }
    }
}

impl Default for GzipCompressor {
    fn default() -> Self {
        GzipCompressor::new(flate2::Compression::default().level())
    }
}

#[derive(Default)]
pub struct NoCompressor;

impl Compress for NoCompressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(from, to)?)
    }
}

impl NoCompressor {
    pub fn new() -> NoCompressor {
        NoCompressor::default()
    }
}

pub struct XzCompressor {
    level: u32,
}

impl XzCompressor {
    pub fn new(level: u32) -> XzCompressor {
        Self { level }
    }
}

impl Compress for XzCompressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = xz2::read::XzEncoder::new(from, self.level);
        Ok(io::copy(&mut encoder, to)?)
    }
}

pub struct ZstdCompressor {
    level: u32,
}

impl ZstdCompressor {
    pub fn new(level: u32) -> ZstdCompressor {
        Self { level }
    }
}

impl Compress for ZstdCompressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = zstd::stream::read::Encoder::new(from, self.level as i32)?;
        Ok(io::copy(&mut encoder, to)?)
    }
}

pub struct Bzip2Compressor {
    level: u32,
}

impl Bzip2Compressor {
    pub fn new(level: u32) -> Self {
        Self { level }
    }
}

impl Compress for Bzip2Compressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = bzip2::read::BzEncoder::new(from, bzip2::Compression::new(self.level));
        Ok(io::copy(&mut encoder, to)?)
    }
}

struct BrotliCompressor {
    quality: u32,
}

impl BrotliCompressor {
    pub fn new(quality: u32) -> BrotliCompressor {
        Self { quality }
    }
}

impl Compress for BrotliCompressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut reader = brotli::CompressorReader::new(from, 4096, self.quality, 21);
        Ok(io::copy(&mut reader, to)?)
    }
}

struct Bzip3Compressor {
    block_size: usize,
}

impl Compress for Bzip3Compressor {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = match bzip3::read::Bz3Encoder::new(from, self.block_size) {
            Ok(d) => d,
            Err(e) => return Err(Error::CompressorError(format!("{}", e))),
        };
        Ok(io::copy(&mut encoder, to)?)
    }
}

impl Bzip3Compressor {
    pub fn new(block_size: usize) -> Self {
        Self { block_size }
    }

    /// Level ranging from 1 to 9 -> block size ranging from 65kiB to 511MiB
    /// block size will be coerced within its valid range when level is outside 1..=9
    fn level_to_block_size(level: u32) -> usize {
        let low = ByteSize::kib(65).0 as usize;
        let up = ByteSize::mib(511).0 as usize;
        let part = (up - low) / 9;

        match level {
            l @ 2..=8 => low + l as usize * part,
            u32::MIN..=1 => low,
            9..=u32::MAX => up,
        }
    }
}

pub struct GzipDecompressor;
pub struct XzDecompressor;
pub struct ZstdDecompressor;
pub struct Bzip2Decompressor;
pub struct NoDecompressor;
pub struct BrotliDecompressor;
pub struct Bzip3Decompressor;

impl Decompress for GzipDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(&mut flate2::read::GzDecoder::new(from), to)?)
    }
}

impl Decompress for XzDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(&mut xz2::read::XzDecoder::new(from), to)?)
    }
}

impl Decompress for ZstdDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(&mut zstd::stream::read::Decoder::new(from)?, to)?)
    }
}

impl Decompress for Bzip2Decompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(&mut bzip2::read::BzDecoder::new(from), to)?)
    }
}

impl Decompress for NoDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(from, to)?)
    }
}

pub struct ExternalFilter<'a> {
    cmd: &'a Vec<String>,
}

impl<'a> ExternalFilter<'a> {
    pub fn new(cmd: &'a Vec<String>) -> Self {
        Self { cmd }
    }

    fn process_filter(args: &Vec<String>, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let cmd = args;
        let mut command = Command::new(&cmd[0]);
        if cmd.len() > 1 {
            command.args(&cmd[1..]);
        }
        command
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = command.spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();

        // TODO: optimize performance (spawning threads frequently)
        // it's really a trouble and hard using unix `poll` here (can't retrieve raw file descriptors)

        // child thread: `from` -> `stdin`
        let from_addr = &from as *const &mut dyn Read as usize;
        let thread = spawn(move || unsafe {
            let mut stdin = stdin;
            let from: &mut dyn Read = transmute_copy(&*(from_addr as *const &mut dyn Read));
            let result = io::copy(from, &mut stdin);
            drop(stdin);
            result
        });

        // main thread: `stdout` -> `to`
        let read_size = io::copy(&mut stdout, to)?;
        let status = child.wait().unwrap();
        if !status.success() {
            return Err(Error::FilterNonZeroExit(status.code().unwrap()));
        }

        thread.join().unwrap()?;
        Ok(read_size)
    }
}

impl<'a> Compress for ExternalFilter<'a> {
    fn compress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Self::process_filter(self.cmd, from, to)
    }
}

impl<'a> Decompress for ExternalFilter<'a> {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Self::process_filter(self.cmd, from, to)
    }
}

impl Decompress for BrotliDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut reader = brotli::Decompressor::new(from, 4096);
        Ok(io::copy(&mut reader, to)?)
    }
}

impl Decompress for Bzip3Decompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut decoder = match bzip3::read::Bz3Decoder::new(from) {
            Ok(d) => d,
            Err(e) => return Err(Error::DecompressorError(format!("{}", e))),
        };
        Ok(io::copy(&mut decoder, to)?)
    }
}
