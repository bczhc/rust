use flate2::read::GzEncoder;
use flate2::Compression;
use std::io;
use std::io::{Read, Write};

use crate::errors::Result;

pub trait Compress {
    fn compress(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64>;
}

pub struct GzipCompressor {
    level: Compression,
}

impl Compress for GzipCompressor {
    fn compress(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        let mut encoder = GzEncoder::new(from, self.level);
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
    fn compress(&self, from: &mut dyn Read, to: &mut dyn Write) -> Result<u64> {
        Ok(io::copy(from, to)?)
    }
}

impl NoCompressor {
    pub fn new() -> NoCompressor {
        NoCompressor::default()
    }
}
