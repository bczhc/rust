use std::io::Write;

pub struct StreamDuplicator<'a, W1, W2>
where
    W1: Write,
    W2: Write,
{
    writer1: &'a mut W1,
    writer2: &'a mut W2,
}

impl<'a, W1, W2> StreamDuplicator<'a, W1, W2>
where
    W1: Write,
    W2: Write,
{
    #[inline]
    pub fn new(writer1: &'a mut W1, writer2: &'a mut W2) -> StreamDuplicator<'a, W1, W2> {
        Self { writer1, writer2 }
    }
}

impl<'a, W1, W2> Write for StreamDuplicator<'a, W1, W2>
where
    W1: Write,
    W2: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // for synchronization I can only use this, to ensure the written data amount is the same...
        self.writer1.write_all(buf)?;
        self.writer2.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer1.flush()?;
        self.writer2.flush()?;
        Ok(())
    }
}
