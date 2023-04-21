use digest::Update;
use std::io;
use std::io::Write;

pub struct HashWriter<H>(pub H)
where
    H: Update;

impl<H> Write for HashWriter<H>
where
    H: Update,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
