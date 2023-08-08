use std::io::{Read, Write};

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

/// A wrapper reader. When read, also write the
/// data to a third writer.
///
/// # Examples
/// ```
/// use std::io::{Cursor, Read};
/// use bczhc_lib::io::duplicator::DuplicationReader;
/// let mut dupe_writer = Cursor::new(Vec::new());
/// let mut data = Cursor::new(*b"hello");
/// let mut reader = DuplicationReader::new(&mut data, &mut dupe_writer);
///
/// let mut content = String::new();
/// reader.read_to_string(&mut content).unwrap();
///
/// assert_eq!(&dupe_writer.into_inner(), b"hello");
/// assert_eq!(content, "hello");
/// ```
pub struct DuplicationReader<R, DW>
where
    R: Read,
    DW: Write,
{
    reader: R,
    dupe_writer: DW,
}

impl<R, DW> DuplicationReader<R, DW>
where
    R: Read,
    DW: Write,
{
    pub fn new(reader: R, duplication_writer: DW) -> Self {
        Self {
            reader,
            dupe_writer: duplication_writer,
        }
    }
}

impl<R, DW> Read for DuplicationReader<R, DW>
where
    R: Read,
    DW: Write,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.reader.read(buf)?;
        self.dupe_writer.write_all(&buf[..size])?;
        Ok(size)
    }
}
