use crate::Compressor;
use flate2::Compression;
use std::io;
use std::io::{BufReader, Read, Write};
use std::str::FromStr;

use crate::errors::Result;

#[derive(Copy, Clone)]
pub enum Level {
    Best,
    Numeric(u32),
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

pub fn create_compressor(method: Compressor, level: Level) -> Box<dyn Compress> {
    let level = match level {
        Level::Best => method.best_level(),
        Level::Numeric(n) => n,
    };

    match method {
        Compressor::Gzip => Box::new(GzipCompressor::new(level)),
        Compressor::Xz => Box::new(XzCompressor::new(level)),
        Compressor::Zstd => Box::new(ZstdCompressor::new(level)),
        Compressor::None => Box::new(NoCompressor::new()),
        Compressor::External => {
            // ensured by clap
            unreachable!()
        }
    }
}

pub fn create_decompressor(method: Compressor) -> Box<dyn Decompress> {
    match method {
        Compressor::Gzip => Box::new(GzipDecompressor),
        Compressor::Xz => Box::new(XzDecompressor),
        Compressor::Zstd => Box::new(ZstdDecompressor),
        Compressor::None => Box::new(NoDecompressor),
        Compressor::External => {
            unreachable!()
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
    level: Compression,
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
            level: Compression::new(level),
        }
    }
}

impl Default for GzipCompressor {
    fn default() -> Self {
        GzipCompressor::new(Compression::default().level())
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

pub struct GzipDecompressor;
pub struct XzDecompressor;
pub struct ZstdDecompressor;
pub struct NoDecompressor;

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

impl Decompress for NoDecompressor {
    fn decompress_to(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(from, to)?)
    }
}
