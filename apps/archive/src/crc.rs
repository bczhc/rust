use crc_lib::{Digest, Width};
use std::io;
use std::io::Write;

pub struct DigestWriter<'a, 'b, W>
where
    W: Width,
{
    digest: &'a mut Digest<'b, W>,
}

impl<'a, 'b> Write for DigestWriter<'a, 'b, u32> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'b> DigestWriter<'a, 'b, u32> {
    pub fn new(digest: &'a mut Digest<'b, u32>) -> DigestWriter<'a, 'b, u32> {
        Self { digest }
    }
}

impl<'a, 'b> Write for DigestWriter<'a, 'b, u64> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'b> DigestWriter<'a, 'b, u64> {
    pub fn new(digest: &'a mut Digest<'b, u64>) -> DigestWriter<'a, 'b, u64> {
        Self { digest }
    }
}
