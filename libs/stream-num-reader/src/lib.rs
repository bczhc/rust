use std::io;
use std::io::Read;
use std::marker::PhantomData;

use crate::read_types::ReadType;

pub mod read_types {
    use std::io;
    use std::io::Read;

    use byteorder::{BE, LE, ReadBytesExt};

    pub trait ReadType {
        type NumType;

        fn read_num<R: Read>(reader: &mut R) -> io::Result<Self::NumType>;
    }

    macro_rules! struct_def {
        ($($name:tt $fn_name:tt $ty:ty),+ $(,)?) => {
            $(
                pub struct $name;

                impl ReadType for $name {
                    type NumType = $ty;

                    #[inline(always)]
                    fn read_num<R: Read>(reader: &mut R) -> io::Result<Self::NumType> {
                        reader.$fn_name()
                    }
                }
            )*
        };
        ($($name:tt $fn_name:tt $endianness:tt $ty:ty),+ $(,)?) => {
            $(
                pub struct $name;

                impl ReadType for $name {
                    type NumType = $ty;

                    #[inline(always)]
                    fn read_num<R: Read>(reader: &mut R) -> io::Result<Self::NumType> {
                        reader.$fn_name::<$endianness>()
                    }
                }
            )*
        };
    }

    struct_def!(S8 read_i8 i8);
    struct_def!(
        S16LE read_i16 LE i16,
        S16BE read_i16 BE i16,
        S32LE read_i32 LE i32,
        S32BE read_i32 BE i32,
        F32LE read_f32 LE f32,
        F32BE read_f32 BE f32,
    );
}

pub struct NumberReader<R: Read, T: ReadType> {
    inner: R,
    phantom: PhantomData<T>,
}

impl<R: Read, T: ReadType> NumberReader<R, T> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            phantom: Default::default(),
        }
    }
}

impl<R: Read, T: ReadType> Iterator for NumberReader<R, T> {
    type Item = io::Result<T::NumType>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = T::read_num(&mut self.inner);
        match result {
            Ok(r) => Some(Ok(r)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::io::Cursor;

    use byteorder::{BE, ByteOrder, LE};

    use crate::NumberReader;
    use crate::read_types;

    #[test]
    fn test1() {
        let data = [1_u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut cursor = Cursor::new(data);
        let mut reader = NumberReader::<_, read_types::S32BE>::new(&mut cursor);
        assert_eq!(
            reader.next().map(Result::unwrap),
            Some(BE::read_i32(&[1, 2, 3, 4]))
        );
        assert_eq!(
            reader.next().map(Result::unwrap),
            Some(BE::read_i32(&[5, 6, 7, 8]))
        );
        assert_eq!(reader.next().map(Result::unwrap), None);
    }

    #[test]
    fn test2() {
        let data = [1_u8, 2, 3, 4, 5, 6];
        let mut cursor = Cursor::new(data);
        let mut reader = NumberReader::<_, read_types::S16LE>::new(&mut cursor);
        assert_eq!(
            reader.next().map(Result::unwrap),
            Some(LE::read_i16(&[1, 2]))
        );
        assert_eq!(
            reader.next().map(Result::unwrap),
            Some(LE::read_i16(&[3, 4]))
        );
        assert_eq!(
            reader.next().map(Result::unwrap),
            Some(LE::read_i16(&[5, 6]))
        );
        assert_eq!(reader.next().map(Result::unwrap), None);
    }
}
