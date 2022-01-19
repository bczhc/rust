use bczhc_lib::io::TryReadExact;
use byteorder::ByteOrder;
use std::io::Read;
use std::marker::PhantomData;

pub struct FileIntReader {}

impl FileIntReader {
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use binary_wav_codec::int_reader::FileIntReader;
    /// use binary_wav_codec::int_reader::int_type;
    /// use byteorder::{LittleEndian, ByteOrder};
    ///
    /// let data = [1_u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    ///
    /// let mut cursor = Cursor::new(&data);
    /// let mut iter = FileIntReader::new_iterator::<int_type::I24, LittleEndian, _>(&mut cursor);
    ///
    /// assert_eq!(iter.next(), Some((LittleEndian::read_i24(&[1, 2, 3]), 3)));
    /// assert_eq!(iter.next(), Some((LittleEndian::read_i24(&[4, 5, 6]), 3)));
    /// assert_eq!(iter.next(), Some((LittleEndian::read_i24(&[7, 8, 9]), 3)));
    /// assert_eq!(iter.next(), Some((LittleEndian::read_i24(&[10, 0, 0]), 1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn new_iterator<I, E, R>(reader: &mut R) -> Ints<I, R, E>
    where
        I: IntReadable<E>,
        E: ByteOrder,
        R: Read,
        [(); I::READ_SIZE]:,
    {
        Ints {
            reader,
            buf: [0_u8; I::READ_SIZE],
            phantom: Default::default(),
            phantom2: Default::default(),
        }
    }
}

pub struct Ints<'a, I, R, E>
where
    I: IntReadable<E>,
    R: Read,
    E: ByteOrder,
    [(); I::READ_SIZE]:,
{
    reader: &'a mut R,
    buf: [u8; I::READ_SIZE],
    phantom: PhantomData<I>,
    phantom2: PhantomData<E>,
}

impl<I, R, E> Iterator for Ints<'_, I, R, E>
where
    I: IntReadable<E>,
    R: Read,
    E: ByteOrder,
    [(); I::READ_SIZE]:,
{
    type Item = (I::ResultType, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.try_read_exact(&mut self.buf);
        match result {
            Ok(size) => {
                if size == 0 {
                    return None;
                }
                if size < I::READ_SIZE {
                    // reach the end of the file, but have insufficient bytes to construct a number
                    // set the remain bytes to zero
                    for i in size..I::READ_SIZE {
                        self.buf[i] = 0_u8;
                    }
                }
                Some((I::read(&self.buf), size))
            }
            Err(e) => {
                panic!("IO error: {}", e);
            }
        }
    }
}

/// `E`: [`byteorder::LittleEndian`] or [`byteorder::BigEndian`]
pub trait IntReadable<E>
where
    E: ByteOrder,
{
    const READ_SIZE: usize;
    type ResultType;
    fn read(buf: &[u8]) -> Self::ResultType;
}

pub mod int_type {
    pub struct I8;
    pub struct U8;
    pub struct I16;
    pub struct U16;
    pub struct I24;
    pub struct U24;
    pub struct I32;
    pub struct U32;
    pub struct I64;
    pub struct U64;
    pub struct I128;
    pub struct U128;
}

impl<E> IntReadable<E> for int_type::I8
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 1;
    type ResultType = i8;

    fn read(buf: &[u8]) -> Self::ResultType {
        buf[0] as i8
    }
}

impl<E> IntReadable<E> for int_type::U8
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 1;
    type ResultType = u8;

    fn read(buf: &[u8]) -> Self::ResultType {
        buf[0]
    }
}

impl<E> IntReadable<E> for int_type::I16
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 2;
    type ResultType = i16;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_i16(buf)
    }
}

impl<E> IntReadable<E> for int_type::U16
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 2;
    type ResultType = u16;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_u16(buf)
    }
}

impl<E> IntReadable<E> for int_type::I24
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 3;
    type ResultType = i32;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_i24(buf)
    }
}

impl<E> IntReadable<E> for int_type::U24
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 3;
    type ResultType = u32;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_u24(buf)
    }
}

impl<E> IntReadable<E> for int_type::I32
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 4;
    type ResultType = i32;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_i32(buf)
    }
}

impl<E> IntReadable<E> for int_type::U32
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 4;
    type ResultType = u32;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_u32(buf)
    }
}

impl<E> IntReadable<E> for int_type::I64
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 8;
    type ResultType = i64;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_i64(buf)
    }
}

impl<E> IntReadable<E> for int_type::U64
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 8;
    type ResultType = u64;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_u64(buf)
    }
}

impl<E> IntReadable<E> for int_type::I128
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 16;
    type ResultType = i128;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_i128(buf)
    }
}

impl<E> IntReadable<E> for int_type::U128
where
    E: ByteOrder,
{
    const READ_SIZE: usize = 16;
    type ResultType = u128;

    fn read(buf: &[u8]) -> Self::ResultType {
        E::read_u128(buf)
    }
}
